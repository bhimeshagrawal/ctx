import * as arrow from "apache-arrow";

export function createDocumentsSchema(): arrow.Schema {
  return new arrow.Schema([
    new arrow.Field("id", new arrow.Utf8(), false),
    new arrow.Field("sourceType", new arrow.Utf8(), false),
    new arrow.Field("sourcePath", new arrow.Utf8(), true),
    new arrow.Field("sourceHash", new arrow.Utf8(), false),
    new arrow.Field("title", new arrow.Utf8(), true),
    new arrow.Field("tags", new arrow.List(new arrow.Field("item", new arrow.Utf8(), true)), false),
    new arrow.Field("createdAt", new arrow.Utf8(), false),
    new arrow.Field("updatedAt", new arrow.Utf8(), false),
    new arrow.Field("metadata", new arrow.Utf8(), false)
  ]);
}

export function createChunksSchema(dimension: number): arrow.Schema {
  return new arrow.Schema([
    new arrow.Field("id", new arrow.Utf8(), false),
    new arrow.Field("documentId", new arrow.Utf8(), false),
    new arrow.Field("chunkIndex", new arrow.Int32(), false),
    new arrow.Field("content", new arrow.Utf8(), false),
    new arrow.Field("contentHash", new arrow.Utf8(), false),
    new arrow.Field("tokenEstimate", new arrow.Int32(), false),
    new arrow.Field(
      "embedding",
      new arrow.FixedSizeList(dimension, new arrow.Field("item", new arrow.Float32(), true)),
      false
    ),
    new arrow.Field("title", new arrow.Utf8(), true),
    new arrow.Field("sourcePath", new arrow.Utf8(), true),
    new arrow.Field("tags", new arrow.List(new arrow.Field("item", new arrow.Utf8(), true)), false),
    new arrow.Field("createdAt", new arrow.Utf8(), false),
    new arrow.Field("metadata", new arrow.Utf8(), false)
  ]);
}

export function createProfilesSchema(): arrow.Schema {
  return new arrow.Schema([
    new arrow.Field("id", new arrow.Utf8(), false),
    new arrow.Field("name", new arrow.Utf8(), false),
    new arrow.Field("defaultTopK", new arrow.Int32(), false),
    new arrow.Field("defaultChunkSize", new arrow.Int32(), false),
    new arrow.Field("defaultChunkOverlap", new arrow.Int32(), false),
    new arrow.Field("outputMode", new arrow.Utf8(), false),
    new arrow.Field("embeddingModel", new arrow.Utf8(), false),
    new arrow.Field("metadata", new arrow.Utf8(), false)
  ]);
}
