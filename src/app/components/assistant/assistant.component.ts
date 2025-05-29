import { CommonModule } from '@angular/common';
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
    CommonModule,
    FormsModule,
    MatButtonModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatListModule,
    ReactiveFormsModule,
  ],
  templateUrl: './assistant.component.html',
  styleUrl: './assistant.component.scss',
})
export class AssistantComponent {
  public messageText: string = '';
  public prePromptText: string = `You have access to a database with the following schema:
    CREATE TABLE electricity_consumption (
    electricity_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_kwh DOUBLE NOT NULL);

    CREATE TABLE gas_consumption (
    gas_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_m3 DOUBLE NOT NULL);

    Are there SQLite queries you want to execute to help you answer this user's intent. If so, reply with JSON fitting the following schema: [{"query": "SQLite query code", "label": "unique label", "comments": "explanation of the query"}]. The user intent follows next.`;
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
