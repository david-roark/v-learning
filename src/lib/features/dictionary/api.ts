import { invoke } from "@tauri-apps/api/core";
import { invokeCmd } from "$lib/shared/tauri";
import type { DictPackInfo, DictionaryEntry } from "$lib/shared/types";

const CHUNK_SIZE = 4 * 1024 * 1024; // 4 MB raw binary chunks

export const dictionaryApi = {
  lookup: (word: string) => invokeCmd<DictionaryEntry>("lookup_word", { word }),
  search: (query: string, limit = 30) =>
    invokeCmd<DictionaryEntry[]>("search_dictionary", { query, limit }),
  count: () => invokeCmd<number>("get_dictionary_count"),
  importTsv: (content: string) => invokeCmd<number>("import_dictionary_tsv", { content }),
  getPackInfo: () => invokeCmd<DictPackInfo>("get_dict_pack_info"),
  installPackFile: async (
    file: File,
    onProgress?: (pct: number) => void,
  ): Promise<DictPackInfo> => {
    await invokeCmd<void>("begin_dict_pack_install");
    let offset = 0;
    while (offset < file.size) {
      const end = Math.min(offset + CHUNK_SIZE, file.size);
      const buf = await file.slice(offset, end).arrayBuffer();
      await invoke("append_dict_pack_chunk", new Uint8Array(buf));
      offset = end;
      onProgress?.(Math.round((offset / file.size) * 100));
    }
    return invokeCmd<DictPackInfo>("finish_dict_pack_install");
  },
  removePack: () => invokeCmd<void>("remove_dict_pack"),
  collocations: (word: string) => invokeCmd<string[]>("fetch_collocations", { word }),
};
