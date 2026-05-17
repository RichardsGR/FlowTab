export interface PageEntry {
  id: number;
  name: string;
  url: string;
  description: string;
  pinyin_full: string;
  pinyin_initials: string;
  created_at: string;
  updated_at: string;
}

export interface SearchResult {
  entry: PageEntry;
  score: number;
  is_open: boolean;
  source: "indexed" | "open_tab";
}

export interface OpenTab {
  url: string;
  title: string;
  window_id: number;
  tab_index: number;
}
