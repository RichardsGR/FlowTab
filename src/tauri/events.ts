import { getCurrentWindow } from "@tauri-apps/api/window";

export async function hideWindow() {
  await getCurrentWindow().hide();
}
