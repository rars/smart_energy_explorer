import { Injectable } from '@angular/core';

import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';

@Injectable({
  providedIn: 'root',
})
export class CsvExportService {
  async exportToCSV(data: any[], defaultFilename: string) {
    const csvRows = [];
    const headers = Object.keys(data[0]);
    csvRows.push(headers.join(','));

    for (const row of data) {
      const values = headers.map((header) => JSON.stringify(row[header] ?? ''));
      csvRows.push(values.join(','));
    }

    const csvString = csvRows.join('\r\n');

    const encoder = new TextEncoder();
    const csvBytes = encoder.encode(csvString);

    const filePath = await save({
      defaultPath: defaultFilename,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (filePath) {
      await writeFile(filePath, csvBytes);
    }
  }
}
