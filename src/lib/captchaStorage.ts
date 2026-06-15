import type { CaptchaResolver } from "@/types";
import { invoke, isTauri } from "./tauri";

export async function listCaptchaResolvers(): Promise<string[]> {
  if (!isTauri()) return [];
  return invoke<string[]>("list_captcha_resolvers");
}

export async function readCaptchaResolver(id: string): Promise<CaptchaResolver | null> {
  if (!isTauri()) return null;
  try {
    const json = await invoke<string>("read_captcha_resolver", { id });
    return JSON.parse(json) as CaptchaResolver;
  } catch {
    return null;
  }
}

export async function writeCaptchaResolver(resolver: CaptchaResolver): Promise<void> {
  if (!isTauri()) return;
  const json = JSON.stringify(resolver, null, 2);
  await invoke("write_captcha_resolver", { id: resolver.id, json });
}

export async function deleteCaptchaResolver(id: string): Promise<void> {
  if (!isTauri()) return;
  await invoke("delete_captcha_resolver", { id });
}
