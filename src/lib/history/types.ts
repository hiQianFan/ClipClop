export type ContentType =
  | "text"
  | "link"
  | "color"
  | "image"
  | "file";

export type SourceApp = { id: string; name: string };
export type HistorySourceOption = SourceApp & { available: boolean };
export type HistoryFacets = {
  type_total: number;
  type_counts: Partial<Record<ContentType, number>>;
  sources: HistorySourceOption[];
};

export type HistoryFilters = {
  content_type: ContentType | null;
  source_id: string | null;
  time_range: "any" | "day" | "week" | "month";
};

export type ClipMetadata = {
  char_count?: number;
  width?: number;
  height?: number;
  files?: string[];
  file_sizes?: Array<number | null>;
};

export type ClipSummary = {
  id: string;
  content_type: ContentType;
  preview: string;
  source_app: SourceApp | null;
  created_at: string;
  last_used_at: string;
  byte_size: number;
  metadata: ClipMetadata;
};

export type FlavorInfo = { format: string; byte_size: number };

export type ClipDetail = ClipSummary & {
  plain_text: string | null;
  flavors: FlavorInfo[];
};

export type HistoryPage = {
  items: ClipSummary[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
};

export type AppError = { code: string };
