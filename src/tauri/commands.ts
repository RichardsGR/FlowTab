import { invoke } from "@tauri-apps/api/core";
import type { PageEntry, SearchResult, OpenTab } from "./types";

export async function searchPages(query: string): Promise<SearchResult[]> {
  return invoke("search_pages", { query });
}

export async function addPage(name: string, url: string, description?: string): Promise<PageEntry> {
  return invoke("add_page", { input: { name, url, description } });
}

export async function updatePage(id: number, name: string, url: string, description?: string): Promise<PageEntry> {
  return invoke("update_page", { id, input: { name, url, description } });
}

export async function deletePage(id: number): Promise<void> {
  return invoke("delete_page", { id });
}

export async function listPages(): Promise<PageEntry[]> {
  return invoke("list_pages");
}

export async function getOpenTabs(browser?: string): Promise<OpenTab[]> {
  return invoke("get_open_tabs", { browser });
}

export async function switchOrOpenTab(url: string, forceNew?: boolean): Promise<string> {
  return invoke("switch_or_open_tab", { url, forceNew });
}

export async function openNewTab(url: string): Promise<void> {
  return invoke("open_new_tab", { url });
}

export async function importBookmarks(): Promise<number> {
  return invoke("import_bookmarks");
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export async function hideSearchWindow(): Promise<void> {
  return invoke("hide_search_window");
}

export async function analyzeHistory(): Promise<number> {
  return invoke("analyze_history");
}

export async function closeTab(url: string): Promise<void> {
  return invoke("close_tab", { url });
}

export async function cleanDuplicatePages(): Promise<number> {
  return invoke("clean_duplicate_pages");
}
