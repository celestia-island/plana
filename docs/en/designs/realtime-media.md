# Realtime Media Protocol — Vendor-Neutral Design

> Status: accepted (P7#A follow-up, 2026-08-05)
> Scope: plana realtime omni-session protocol — canonical event vocabulary,
> media/audio waveform formats, and the streaming transport layering.
> The gateway (arona) is the ONLY place that knows vendor specifics.

## 1. Problem

The first realtime PR (plana #172) used the OpenAI-Realtime event vocabulary
directly on the wire (`input_audio_buffer.append`, `response.audio.delta`,
`speech_started`, …). That couples every client to one vendor's names:

- OpenAI Realtime: `session.update` / `input_audio_buffer.*` /
  `response.output_audio.delta` (GA) — beta names `response.audio.delta`.
- Qwen-Omni-Realtime: OpenAI **beta** vocabulary (~95% identical) + Qwen-only
  `input_image_buffer.append`, `semantic_vad`, `enable_search`.
- Gemini Live: completely different envelope (`setup` / `realtimeInput` /
  `serverContent` / `toolCall`, no session object).

Clients (chest webui, arona Playground) must not know any of this. The
canonical protocol lives in plana; vendor dialects are translated ONLY in the
arona gateway adapters. This document fixes the canonical vocabulary, the
audio waveform contract, and the streaming transport so that the upcoming
video/character-performance path (LPM-class) does not fork the wire format.

## 2. Canonical event vocabulary

Events are tagged unions (`"type": "…"`), snake_case, vendor-neutral names.
Vendors never see these names; adapters map canonical ↔ vendor 1:1.

### 2.1 Client → gateway (`RealtimeClientEvent`)

| Canonical | Payload | Vendor mapping |
|---|---|---|
| `session.configure` | `RealtimeSessionConfig` | OpenAI `session.update`; Qwen `session.update`; Gemini `setup` |
| `audio.input.append` | `RealtimeAudioChunk` | OpenAI/Qwen `input_audio_buffer.append`; Gemini `realtimeInput.audio` |
| `audio.input.commit` | — | OpenAI/Qwen `input_audio_buffer.commit`; Gemini `clientContent.turnComplete` |
| `audio.input.clear` | — | OpenAI/Qwen `input_audio_buffer.clear`; Gemini `realtimeInput.audioStreamEnd` + discard |
| `video.input.frame` | `RealtimeVideoFrame` | Qwen `input_image_buffer.append`; Gemini `realtimeInput.video`; OpenAI via conversation item |
| `response.request` | — | OpenAI/Qwen `response.create`; Gemini `clientContent` (new turn) |
| `response.cancel` | — | OpenAI/Qwen `response.cancel`; Gemini `activityStart` interrupt |
| `session.close` | — | OpenAI/Qwen `session.stop`; Gemini `goAway` ack |

### 2.2 Gateway → client (`RealtimeServerEvent`)

| Canonical | Payload | Vendor mapping |
|---|---|---|
| `session.created` | config | OpenAI/Qwen `session.created`; Gemini `setupComplete` |
| `session.configured` | config | OpenAI/Qwen `session.updated`; Gemini — |
| `turn.speech_started` | `audio_start_ms` | OpenAI/Qwen `input_audio_buffer.speech_started`; Gemini `serverContent.interrupted` |
| `turn.speech_stopped` | `audio_end_ms` | OpenAI/Qwen `input_audio_buffer.speech_stopped`; Gemini `serverContent.turnComplete` |
| `response.started` | `response_id` | OpenAI/Qwen `response.created`; Gemini `serverContent` first part |
| `response.audio.delta` | `RealtimeAudioChunk` | OpenAI/Qwen `response.audio.delta`; Gemini `serverContent` inlineData audio |
| `response.audio.end` | `response_id` | OpenAI/Qwen `response.audio.done`; Gemini — |
| `response.transcript.delta` | text | OpenAI/Qwen `response.audio_transcript.delta`; Gemini `outputTranscription` |
| `response.text.delta` | text | OpenAI/Qwen `response.text.delta`; Gemini `serverContent` text part |
| `response.video.frame` | `RealtimeVideoFrame` | Qwen/OpenAI gen output frames; LPM-class engines |
| `response.done` | `response_id` + `usage` | OpenAI/Qwen `response.done`; Gemini `usageMetadata` |
| `error` | code+message | all vendors |

**Rust naming stays semantic** (`InputAudioBufferAppend` etc. are fine as
variants); the serde tag is the canonical name above. Breaking change vs
plana #172: only the `type` strings change (`input_audio_buffer.append` →
`audio.input.append`, …).

## 3. Audio waveform contract

### 3.1 Canonical codec: PCM16 LE, mono

| Leg | Sample rate | Channels | Format |
|---|---|---|---|
| client → model (uplink) | **16 kHz** | 1 (mono) | PCM16 little-endian |
| model → client (downlink) | **24 kHz** | 1 (mono) | PCM16 little-endian |

Rationale: every vendor converges on exactly this pair (OpenAI 16k in/24k
out, Qwen 16k in/24k out, Gemini 16k in/24k out). No gateway-side
resampling or transcoding is needed — bytes pass through. The client is
responsible for browser-side conversion (webm/opus → PCM16 16k on uplink,
PCM16 24k → WebAudio on downlink).

`RealtimeAudioChunk` carries `mime` (`audio/pcm`), `sample_rate` and
`data_base64` explicitly — the chunk is self-describing so a future codec
change (e.g. Opus) does not break old decoders.

### 3.2 Frame pacing

- Uplink chunks: 100 ms (≈ 3200 B at 16 kHz) is the reference size — Qwen's
  official example uses it; the browser MediaRecorder 250 ms slices are
  decoded/resampled per-chunk, so sub-100 ms delivery is achievable.
- Downlink chunks: vendor-driven; clients MUST buffer per-chunk and schedule
  playback contiguously (the chest player queues deltas against the audio
  context clock).
- Barge-in: `turn.speech_started` is authoritative — client stops playback,
  drops queued deltas, and the gateway forwards the upstream's speech-start
  immediately (it already does; keep it on the hot path).

### 3.3 Future codecs (explicitly allowed, not breaking)

The chunk is self-describing; a future `opus` mime with an `encoding` hint is
a new chunk shape, not a protocol change. Vendors that add Opus on the WS
path (none today) would be negotiated via `session.configure`.

## 4. Streaming transport layering

This is where "realtime" becomes media streaming. Three layers:

```
L2  Application  — canonical events (JSON, tagged unions)      [plana types]
L1  Transport    — JSON-RPC over WebSocket; BINARY frames for
                   large media payloads                        [realtime.rs + engine.rs]
L0  Physical     — TLS WebSocket (or WebRTC later)             [gateway]
```

### 4.1 Base64-in-JSON (current, v1)

`data_base64` in JSON. Cost: +33 % bandwidth/CPU. Fine for ≤100 ms audio
chunks (~4 kB → ~5.5 kB) and ≤15 fps JPEG frames (~50–200 kB → 67–270 kB).
This is the compatibility baseline; every client and engine can speak it
with zero new machinery.

### 4.2 Binary frames (v2, streaming scale)

Reuse the **CEP binary-transfer triple** already implemented in arona
(`Engine.BinaryStart` announce → raw WS binary frames → `Engine.BinaryEnd`),
flipped to the client-facing channel:

- Announcer (gateway) sends `binary.start {transfer_id, mime, total_bytes,
  chunk_count, checksum?, stream_id?}` — MIME tags every payload the same
  way CEP does.
- Payload: raw WS binary frames (≤256 KiB each).
- Finisher sends `binary.end {transfer_id, bytes_received, checksum_ok}`.
- Client (browser) side: `WebTransport.binaryType = "arraybuffer"` +
  `onBinary` handlers already exist in chest's transport; the same
  announce/finish JSON-RPC notifications ride the session channel.

Binary mode is for **downlink video frames / large audio bursts** and
**uplink pre-encoded audio** (opus from MediaRecorder when the engine
accepts it). The event vocabulary is unchanged — the same canonical events
flow, but `data_base64` is omitted and bytes travel as binary frames.

### 4.3 Video output (LPM-class, v3)

- ≤15 fps JPEG/WebP frames: binary frames per frame (`response.video.frame`
  event announces each frame's metadata; bytes via binary frames).
- ≥15 fps or 720p: fragmented MP4 (H.264) segments over the same binary
  channel → MSE `SourceBuffer` in the browser. Chunk shape:
  `binary.start {mime: video/mp4; codecs: avc1.42E01E}` + fMP4 segments +
  `binary.end`. No new events — `response.video.frame` carries the segment
  metadata (seq, pts, keyframe flag).

### 4.4 WebRTC (explicitly deferred)

Not needed for a single-viewer chat pane; binary frames over the existing
WS channel cover the latency budget (<300 ms target). WebRTC/SFU is a
product-scale decision (multi-party, TURN), not a protocol decision — the
canonical event layer survives transport swaps unchanged.

## 5. Gateway adapter contract (arona)

- `CloudRealtimeUpstream` (OpenAI/Qwen) and `GeminiRealtimeUpstream` are the
  ONLY places where vendor names appear.
- Each adapter implements canonical → vendor and vendor → canonical mapping
  tables (§2).
- Canonical events are what the RPC layer (`realtime.*`) and the webui
  consume; a vendor-specific field (e.g. Qwen `enable_search`) rides in
  `session.configure` as an opaque `extras: Value` that adapters pass
  through when the vendor supports it.

## 6. Compatibility notes

- plana #172 shipped `input_audio_buffer.append` etc. — this document
  renames the serde tags to §2 names. Both arona (#219) and chest (#219)
  must bump the plana rev and use the new canonical names in adapters /
  store handlers. No other consumers exist yet, so this is a clean break.
- `RealtimeSessionConfig` keeps `input_audio_format`/`output_audio_format`
  as strings (`"pcm16"`) for forward-compat, but the canonical value is
  fixed by §3.1; the chunk carries the authoritative `sample_rate`.
