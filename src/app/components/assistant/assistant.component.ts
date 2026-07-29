import { CommonModule } from '@angular/common';
import { Component, inject, OnInit, signal } from '@angular/core';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatListModule } from '@angular/material/list';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSelectModule } from '@angular/material/select';

import { AssistantService } from '../../services/assistant/assistant.service';
import { ChartComponent } from '../chart/chart.component';

type Message = {
  id: number,
  user: string;
  text: string;
  chartConfiguration: unknown | null;
};

@Component({
  selector: 'app-assistant',
  imports: [
    ChartComponent,
    CommonModule,
    FormsModule,
    MatButtonModule,
    MatCardModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatListModule,
    MatProgressSpinnerModule,
    MatSelectModule,
    ReactiveFormsModule,
  ],
  templateUrl: './assistant.component.html',
  styleUrl: './assistant.component.scss',
})
export class AssistantComponent implements OnInit {
  public messageText: string = '';
  public prePromptText: string = `You are a highly specialized AI assistant focused solely on generating accurate SQLite queries based on user intent. Your responses must be complete, concise, and directly derivable from the provided database schema, avoiding any hypothetical or non-existent tables/columns.

You have access to the following database schema:

-- Stores half-hourly electricity consumption readings in watt-hours.
CREATE TABLE electricity_consumption (
  electricity_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  timestamp DATETIME NOT NULL UNIQUE,
  energy_consumption_wh BIGINT NOT NULL,
  london_date_id INTEGER
);

timestamp is a UTC timestamp without a timezone.
london_date_id is a special integer that corresponds to a date in the format YYYYMMDD, i.e. 20260626 is the 26th June 2026.

The user is interested in answers related to the London timezone, so london_date_id is a column that is useful for finding data related to particular days.

Prefer to report energy consumption in units of kWh rather than Wh unless Wh is specifically requested.

The data looks like this:
[
	{
		"electricity_consumption_id" : 1,
		"timestamp" : "2026-06-26 00:00:00",
		"energy_consumption_wh" : 64,
		"london_date_id" : 20260626
	},
	{
		"electricity_consumption_id" : 2,
		"timestamp" : "2026-06-26 00:30:00",
		"energy_consumption_wh" : 90,
		"london_date_id" : 20260626
	},
	{
		"electricity_consumption_id" : 3,
		"timestamp" : "2026-06-26 01:00:00",
		"energy_consumption_wh" : 59,
		"london_date_id" : 20260626
	}
]

-- Stores half-hourly gas consumption readings in watt-hours.
CREATE TABLE gas_consumption (
    gas_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_wh BIGINT NOT NULL,
    london_date_id INTEGER
);

Both \`electricity_consumption\` and \`gas_consumption\` tables record energy consumption data at precise 30-minute intervals detailing energy consumed within the preceding 30 minute period. In order to determine energy consumption on larger intervals, you can sum the consumption over a range to arrive at the total. Assume \`timestamp\` values are stored in UTC.

Your goal is to identify and provide all necessary SQLite queries to fulfill the user's request. Your response must be a valid JSON object. Do not include any additional text or formatting outside of the JSON. The JSON schema for each query object is as follows:

{"queries":[
  {
    "query": "SQLite query code using only highly compatible SQLite expressions",
    "label": "A concise, unique identifier for the query (e.g., 'total_kwh_july_2023', 'avg_gas_daily_last_week')",
    "comments": "A brief explanation of what the query achieves and why it's relevant to the user's intent."
  }
]}

If the user's intent cannot be fulfilled with any SQLite queries based on the provided schema, you must respond with \`{"queries":[]}\`. Only generate SELECT queries.

Here are some examples of user intents and their expected outputs:

---
User Intent: What was the total electricity consumption in Wh for July 2023?

Expected Output:
{"queries":[
  {
    "query": "SELECT SUM(energy_consumption_wh) FROM electricity_consumption WHERE timestamp >= '2023-07-01 00:00:00' AND timestamp < '2023-08-01 00:00:00';",
    "label": "total_wh_july_2023",
    "comments": "Calculates the sum of electricity consumption in Wh for all entries within July 2023."
  }
]}
---
User Intent: Show me the average daily gas consumption in Wh for the last week.

Expected Output:
{"queries":[
  {
    "query": "SELECT DATE(timestamp), AVG(energy_consumption_wh) FROM gas_consumption WHERE timestamp >= DATETIME('now', '-7 days') GROUP BY DATE(timestamp);",
    "label": "avg_gas_daily_last_week",
    "comments": "Computes the average daily gas consumption in Wh for the past 7 days, grouping results by date."
  }
]}
---
User Intent: Tell me about the weather forecast for tomorrow.

Expected Output:
{"queries":[]}
---

User Intent:`;
  public messages = signal<Message[]>([
    {
      id: 0,
      user: 'Assistant',
      text: 'Hello, how can I help?',
      chartConfiguration: null,
    },
  ]);

  public availableModels = signal<string[]>([]);
  public selectedModel: string = '';
  public status = signal('');
  public isMessageInFlight = signal(false);
  private readonly assistant = inject(AssistantService);

  public constructor() {
    this.assistant.unsubscribeAllListeners();
    this.assistant
      .addListener('assistant-update', (event: any) => {
        this.status.set(event.payload);
      })
      .then(() => console.log('listener added'));
  }

  public ngOnInit(): void {
    this.loadModels();
  }

  public async loadModels(): Promise<void> {
    const models = await this.assistant.getModels();
    this.availableModels.set(models);
    if (models.length > 0 && !this.selectedModel) {
      this.selectedModel = models[0];
    }
  }

  public sendMessage(): void {
    if (this.messageText.trim()) {
      this.messages.update(value => {
        return [...value, {
          id: value.length,
          user: 'You',
          text: this.messageText,
          chartConfiguration: null,
        }];
      });


      this.isMessageInFlight.set(true);

      // Here you would call your chat assistant service/API
      this.assistant
        .ask(this.messageText, this.prePromptText, this.selectedModel)
        .then((response) => {
          console.log(response);
          this.messages.update(value => [...value, {
            id: value.length,
            user: 'Assistant',
            text: response.answer.text,
            chartConfiguration: response.answer.chartConfiguration,
          }]);
        })
        .catch((err) => {
          console.error(err);
        })
        .finally(() => {
          this.isMessageInFlight.set(false);
        });

      this.messageText = '';
    }
  }
}
