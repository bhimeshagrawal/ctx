import type { Table } from "@lancedb/lancedb";
import type { DocumentRow } from "../../types/document.js";

export class DocumentsRepository {
  constructor(private readonly table: Table) {}

  async add(row: DocumentRow): Promise<void> {
    await this.table.add([row]);
  }

  async getById(id: string): Promise<DocumentRow | null> {
    const rows = await this.table.query().where(`id = '${escapeSql(id)}'`).limit(1).toArray();
    return (rows[0] as DocumentRow | undefined) ?? null;
  }
}

function escapeSql(value: string): string {
  return value.replaceAll("'", "''");
}
