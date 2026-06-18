import { Component, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import {
  FieldContext,
  FormField,
  FormRoot,
  disabled,
  form,
  required,
  validate,
} from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { RouterLink } from '@angular/router';

import { MqttService } from '../../services/mqtt/mqtt.service';

const leadingOrTrailingWhitespaceValidator = (ctx: FieldContext<string>) => {
  const val = ctx.value() || '';
  const isTrimmable = val.trim().length !== val.length;
  return isTrimmable
    ? {
        kind: 'leadingOrTrailingWhitespace',
        message: 'Must not contain leading or trailing whitespace',
      }
    : null;
};

interface MqttSettingsForm {
  hostname: string;
  topic: string;
  gasTopic: string;
  username: string;
  password: string;
}

@Component({
  selector: 'app-mqtt-settings',
  imports: [
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    RouterLink,
    FormField,
    FormRoot,
  ],
  templateUrl: './mqtt-settings.component.html',
  styleUrl: './mqtt-settings.component.scss',
})
export class MqttSettingsComponent {
  protected readonly mqttSettings = signal<MqttSettingsForm>({
    hostname: '',
    topic: '',
    gasTopic: '',
    username: '',
    password: '',
  });

  protected readonly isSaving = signal(false);

  protected readonly form = form(
    this.mqttSettings,
    (path) => {
      required(path.hostname, { message: 'Hostname is required' });
      validate(path.hostname, leadingOrTrailingWhitespaceValidator);

      required(path.topic, { message: 'Electricity Sensor Topic is required' });
      validate(path.topic, leadingOrTrailingWhitespaceValidator);

      required(path.gasTopic, { message: 'Gas Sensor Topic is required' });
      validate(path.gasTopic, leadingOrTrailingWhitespaceValidator);

      required(path.username, { message: 'Username is required' });
      validate(path.username, leadingOrTrailingWhitespaceValidator);

      required(path.password, { message: 'Password is required' });
      validate(path.password, leadingOrTrailingWhitespaceValidator);

      disabled(path, { when: () => this.isSaving() });
    },
    {
      submission: {
        action: async () => {
          this.isSaving.set(true);
          try {
            const { hostname, topic, gasTopic, username, password } =
              this.mqttSettings();
            await this.mqttService.saveMqttSettings(
              hostname,
              topic,
              gasTopic,
              username,
              password,
            );
            this.form().reset();
          } finally {
            this.isSaving.set(false);
          }
        },
      },
    },
  );

  public constructor(private readonly mqttService: MqttService) {
    this.mqttService
      .getMqttSettings()
      .pipe(takeUntilDestroyed())
      .subscribe((settings) => {
        this.mqttSettings.set(settings);
      });
  }

  public async clear(): Promise<void> {
    this.isSaving.set(true);
    try {
      await this.mqttService.resetMqttSettings();
      this.mqttSettings.set({
        hostname: '',
        topic: '',
        gasTopic: '',
        username: '',
        password: '',
      });
      this.form().reset();
    } finally {
      this.isSaving.set(false);
    }
  }
}
