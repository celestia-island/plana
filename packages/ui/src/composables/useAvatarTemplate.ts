import md5 from "md5";

async function hashStr(algo: string, str: string): Promise<string> {
  if (!str) return "";
  try {
    const buf = new TextEncoder().encode(str);
    const hash = await crypto.subtle.digest(algo, buf);
    return Array.from(new Uint8Array(hash))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  } catch {
    return "";
  }
}

export async function renderAvatarTemplate(
  template: string,
  email: string,
  username: string,
): Promise<string> {
  let url = template;

  const md5Email = email ? md5(email) : "";
  const sha256Email = await hashStr("SHA-256", email);
  const sha512Email = await hashStr("SHA-512", email);
  const sha256Username = await hashStr("SHA-256", username);
  const sha512Username = await hashStr("SHA-512", username);
  const sha3_256Email = await hashStr("SHA-3-256", email);
  const sha3_512Email = await hashStr("SHA-3-512", email);

  url = url.replace(/\{\{\s*md5_email\s*\}\}/g, md5Email);
  url = url.replace(/\{\{\s*sha256_email\s*\}\}/g, sha256Email);
  url = url.replace(/\{\{\s*sha512_email\s*\}\}/g, sha512Email);
  url = url.replace(/\{\{\s*sha3_256_email\s*\}\}/g, sha3_256Email || sha256Email);
  url = url.replace(/\{\{\s*sha3_512_email\s*\}\}/g, sha3_512Email || sha512Email);
  url = url.replace(/\{\{\s*sha256_username\s*\}\}/g, sha256Username);
  url = url.replace(/\{\{\s*sha512_username\s*\}\}/g, sha512Username);
  url = url.replace(/\{\{\s*email\s*\}\}/g, email);
  url = url.replace(/\{\{\s*username\s*\}\}/g, username);
  url = url.replace(/\{\{\s*lower_username\s*\}\}/g, username.toLowerCase());
  url = url.replace(/\{\{\s*upper_username\s*\}\}/g, username.toUpperCase());
  url = url.replace(/\{\{\s*url_encode_email\s*\}\}/g, encodeURIComponent(email));
  url = url.replace(/\{\{\s*url_encode_username\s*\}\}/g, encodeURIComponent(username));
  url = url.replace(/\{\{\s*base64_email\s*\}\}/g, btoa(unescape(encodeURIComponent(email))));
  url = url.replace(/\{\{\s*base64_username\s*\}\}/g, btoa(unescape(encodeURIComponent(username))));

  return url;
}
