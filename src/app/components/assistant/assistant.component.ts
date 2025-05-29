
import { Component } from '@angular/core';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatListModule } from '@angular/material/list';

import { AssistantService } from '../../services/assistant/assistant.service';

@Component({
  selector: 'app-assistant',
  imports: [
    FormsModule,
    MatButtonModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatListModule,
    ReactiveFormsModule
],
  templateUrl: './assistant.component.html',
  styleUrl: './assistant.component.scss',
})
export class AssistantComponent {
  public messageText: string = '';
  public prePromptText: string = `The following are facts:

    1. You have access to a SQLite database with the following schema:

    \`\`\`sql
    CREATE TABLE electricity_consumption (
      electricity_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
      timestamp DATETIME NOT NULL UNIQUE,
      energy_consumption_kwh DOUBLE NOT NULL
    );

    CREATE TABLE gas_consumption (
      gas_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
      timestamp DATETIME NOT NULL UNIQUE,
      energy_consumption_m3 DOUBLE NOT NULL
    );
    \`\`\`

    2. The consumption data is measured at timestamps with 30-minute intervals.

    3. Queries must be valid SQLite queries with no errors, a high degree of compatibility and can only use the tables and columns mentioned above.

Task:
After the user states their intent or question, if there are SQLite queries you want to execute to help answer the user's intent, reply with a JSON array fitting the following schema:

\`\`\`json
[
  {
    "query": "SQLite query code",
    "label": "unique label",
    "comments": "explanation of the query"
  }
]
\`\`\`

If no queries are needed, return an empty list.

User intent follows next.`;
  public messages: { user: string; text: string }[] = [
    { user: 'Assistant', text: 'Hello, how can I help?' },
  ];

  public constructor(private readonly assistant: AssistantService) {}

  public sendMessage(): void {
    if (this.messageText.trim()) {
      this.messages.push({ user: 'You', text: this.messageText });

      // Here you would call your chat assistant service/API
      this.assistant
        .ask(this.messageText, this.prePromptText)
        .then((response) => {
          this.messages.push({ user: 'Assistant', text: response });
        });

      this.messageText = '';
    }
  }
}
