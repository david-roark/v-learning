import { invokeCmd } from "$lib/shared/tauri";
import type { DictionaryEntry, DictionaryPack, ImportReport } from "$lib/shared/types";

export const dictionaryApi = {
  lookup: (word: string) => invokeCmd<DictionaryEntry>("lookup_word", { word }),
  search: (query: string, limit = 30) =>
    invokeCmd<DictionaryEntry[]>("search_dictionary", { query, limit }),
  count: () => invokeCmd<number>("get_dictionary_count"),
  importTsv: (content: string) => invokeCmd<number>("import_dictionary_tsv", { content }),
  listPacks: () => invokeCmd<DictionaryPack[]>("list_dictionary_packs"),
  downloadPack: (packId: string) => invokeCmd<ImportReport>("download_and_import_pack", { packId }),
  collocations: (word: string) => invokeCmd<string[]>("fetch_collocations", { word }),
};
