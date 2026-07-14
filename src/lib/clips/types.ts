export type ContentType =
  | "text"
  | "link"
  | "color"
  | "code"
  | "image"
  | "file";

export type SourceApp = { id: string; name: string };

export type ClipSummary = {
  id: string;
  content_type: ContentType;
  preview: string;
  source_app: SourceApp | null;
  created_at: string;
  byte_size: number;
  metadata: Record<string, unknown>;
};

export type FlavorInfo = { format: string; byte_size: number };

export type ClipDetail = ClipSummary & {
  plain_text: string | null;
  flavors: FlavorInfo[];
};

export type ClipPage = {
  items: ClipSummary[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
};

export type AppError = { code: string; message: string };
