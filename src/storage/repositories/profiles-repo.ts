import type { Table } from "@lancedb/lancedb";
import type { ProfileRow } from "../../types/profile.js";

export class ProfilesRepository {
  constructor(private readonly table: Table) {}

  async ensureDefault(profile: ProfileRow): Promise<void> {
    const count = await this.table.countRows();
    if (count > 0) {
      return;
    }
    await this.table.add([profile]);
  }
}
