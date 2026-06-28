+++
title = "TIA Portal の前提準備 —— evernight への接続"
description = """TIA Portal で S7-1200/1500 PLC を一度だけ設定し、evernight が接続・自己組網・デバイスの読み書きを、以後すべて自動で行えるようにする手順。"""
lang = "ja"
category = "guides"
subcategory = "router"
+++

# TIA Portal の前提準備 —— evernight へ接続する

> **目的**：TIA Portal で西門子 S7-1200/1500 PLC を**一度だけ**設定し、
> evernight が接続・自己組網・読み書きを**以後すべて自動で**行えるようにする。
> これは CPU プロパティの一度きりの設定であり、ラダー/SCL プログラムロジック
> には一切触れない。

evernight は**2 つのチャネル**で西門子 PLC に接続する。PLC が公開しているもの
に応じて選ぶ：

| チャネル | ポート | アクセス方式 | 必要な TIA 準備 | 適用 |
|---------|--------|-------------|----------------|------|
| **S7comm** | 102 | M / I / Q / DB の生バイト R/W | PUT/GET + 非最適化 DB | レガシー、軽量、OPC UA ライセンス不要 |
| **OPC UA** | 4840 | シンボリック、自己記述 | 内蔵 server の有効化 | **推奨** —— 自動発見、最適化 DB も OK |

OPC UA を有効化できるなら優先する：evernight がシンボルアドレス空間全体を
**自動ブラウズ**し、シンボルの手入力がゼロになる。

---

## パス A —— S7comm（生レジスタアクセス）

### A.1 PUT/GET 通信を有効化

S7-1200/1500 は外部 S7 読み書きを既定で拒否する。

1. **TIA Portal** でプロジェクトを開く。
2. デバイス/ネットワークビューで **CPU をクリック**。
3. プロパティ → **保護とセキュリティ（Protection & Security）→ 接続メカニズム（Connection mechanisms）**。
4. **「リモートパートナからの PUT/GET 通信によるアクセスを許可（Permit access with PUT/GET communication from remote partner）」** にチェック。
5. ハードウェア構成を CPU にダウンロード。

### A.2 対象 DB を非最適化に

最適化ブロックアクセス（S7-1200/1500 の既定）には固定バイトオフセットがなく、絶対アドレス読み出しは失敗する。evernight が読み書きする各 DB について：

1. DB を右クリック → **プロパティ（Properties）**。
2. **「最適化ブロックアクセス（Optimized block access）」** のチェックを外す。
3. 再コンパイルしてダウンロード。

> M マーカと I/Q プロセスイメージは常にバイトアドレス指定可能 —— 変更不要。この
> 手順は DB のみ対象。

### A.3 evernight から接続

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500：`rack=0, slot=1`
- S7-300：`slot=2`

コードで自己組網：

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures が読み取り可能な各 DB を記述
```

---

## パス B —— OPC UA（推奨）

### B.1 ファームウェアとライセンスの前提

- CPU ファームウェア **V2.0+**（OPC UA メソッドは V2.5+）。
- CPU クラスに合う **SIMATIC OPC UA ランタイムライセンス**（CPU プロパティ →
  **ランタイムライセンス（Runtime licenses）→ OPC UA** で割り当て、コンプラ
  イアンス上必須）。

### B.2 OPC UA server を有効化

1. ネットワーク/デバイスビューで **CPU をクリック**。
2. プロパティ → **OPC UA → 一般（General）**：server 名を入力。
3. プロパティ → **OPC UA → サーバ（Server）**：**「OPC UA サーバを有効化（Activate OPC UA server）」** にチェック。
4. クライアントが到達する **PROFINET インターフェース**に server を割り当て。

### B.3 シンボル変数を公開

- **OPC UA → サーバ → サーバインターフェース（Server interface）**：**「標準
  SIMATIC サーバインターフェース（Standard SIMATIC server interface）」** を選
  択し、全シンボルタグ/DB（最適化 DB 含む）を自動公開。

### B.4 認証とセキュリティ

- **OPC UA → サーバ → ユーザ認証（User authentication）**：匿名（信頼できる
  LAN）またはユーザ名/パスワード。
- **OPC UA → サーバ → セキュリティ（Security）**：ポリシーを選択。初回は
  `None` が最も簡単。本番は `Sign & Encrypt`。
- ファームウェア **V3.1+ かつ TIA V19+** では、**「OPC UA サーバアクセス（OPC
  UA server access）」** 機能ロール/ランタイム権限を接続ユーザに付与。

### B.5 クライアント証明書を信頼

OPC UA クライアントは接続時に X.509 証明書を提示し、PLC は未知の証明書を隔離す
る。evernight の初回接続後：

1. TIA Portal → **CPU → 証明書（Certificates）**（オンライン）、**または**
2. PLC **Web サーバ** → "Communication with OPC UA clients"、**または**
3. CPU ディスプレイの証明書マネージャ。

evernight のクライアント証明書を**承認/信頼**する。

### B.6（任意）OPC UA NodeSet XML をエクスポート

NodeSet ファイルは全変数のオフラインマップで、実接続なしの事前計画に便利：

1. CPU プロパティ → **OPC UA → サーバ → エクスポート（Export）**。
2. **「OPC UA XML ファイルをエクスポート（Export OPC UA XML file）」** をクリッ
   ク、`*.Opc.Ua.NodeSet2.xml` を保存。

### B.7 ダウンロード

**ハードウェア構成**をダウンロード。これらは CPU プロパティであり、プログラム
ロジックではない —— ラダー/SCL コードは無傷。

### B.8 evernight から接続

エンドポイント URL：

```
opc.tcp://192.168.1.10:4840
```

evernight は OPC UA クライアントとして接続し、シンボルツリー全体を**ブラウズ**
し、名前で読み書きする —— 手動シンボル入力ゼロ、最適化 DB もそのままアクセス可。

---

## 接続性の検証（ゼロリスクプローブ）

出力を駆動する前に、読み取り専用プローブでチャネルの生存を確認：

```bash
# ポート 102 で S7comm を話しているか？
evernight probe 192.168.1.10 --ports 102

# ポート 4840 で OPC UA server は立っているか？
evernight probe 192.168.1.10 --ports 4840
```

どちらも受動的なハンドシェイク —— 何も読まず何も書かない。

---

## 安全の境界

- 安全インタロック（非常停止、リミット、過負荷）を S7/OPC UA 経路に**絶対に**
  置かないこと。PLC スキャン内に留める。ネットワーク切断で安全機能が無効になっ
  てはならない。
- evernight 制御は**低速**負荷（バルブ、状態機械、モード切替）向け。S7/OPC UA
  の往復遅延は約 10–50 ms —— 監督レベルは十分、モーション/サーボは遅すぎる。
- Q 出力を直接書くより、既存 PLC ロジックが作用する**コマンド M ビット**を書く
  方を優先（トリガ源を乗っ取る）。

---

## トラブルシューティング

| 現象 | 可能な原因 | 対処 |
|------|-----------|------|
| S7 接続拒否 / COTP 確認なし | PUT/GET 無効；rack/slot 誤り；ファイアウォール | A.1；`rack=0 slot=1`（1200/1500）を確認 |
| DB 読みが "optimized" / InvalidAddress を返す | 最適化ブロックアクセスが有効 | A.2 —— 最適化を解除し再コンパイル |
| OPC UA エンドポイント到達不能 | server 未有効化；未ダウンロード；ライセンス不足 | B.2 / B.7 / B.1 |
| OPC UA 接続後すぐ拒否 | クライアント証明書が未信頼 | B.5 |
| ブラウズ結果が空 | 標準 SIMATIC インターフェース未启用 | B.3 |

---

## 参考

- [OPC UA server の有効化（S7-1500）— STEP 7 V20 ドキュメント](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [OPC UA server へのアクセス（エンドポイント URL / ポート 4840）](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [OPC UA XML ファイルのエクスポート（NodeSet）](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
