import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { RouterLink } from '@angular/router';

import { noLeadingOrTrailingWhitespaceValidator } from '../../common/validators';
import { MqttService } from '../../services/mqtt/mqtt.service';

@Component({
  selector: 'app-mqtt-settings',
  imports: [
    CommonModule,
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    ReactiveFormsModule,
    RouterLink,
  ],
  templateUrl: './mqtt-settings.component.html',
  styleUrl: './mqtt-settings.component.scss',
})
export class MqttSettingsComponent {
  public form: FormGroup;

  public constructor(
    private readonly fb: FormBuilder,
    private readonly mqttService: MqttService,
  ) {
    this.form = this.fb.group({
      hostname: [
        '',
        [Validators.required, noLeadingOrTrailingWhitespaceValidator()],
      ],
      topic: [
        '',
        [Validators.required, noLeadingOrTrailingWhitespaceValidator()],
      ],
      username: [
        '',
        [Validators.required, noLeadingOrTrailingWhitespaceValidator()],
      ],
      password: [
        '',
        [Validators.required, noLeadingOrTrailingWhitespaceValidator()],
      ],
    });

    this.mqttService
      .getMqttSettings()
      .pipe(takeUntilDestroyed())
      .subscribe(({ hostname, topic, username, password }) => {
        this.form.get('hostname')?.setValue(hostname);
        this.form.get('topic')?.setValue(topic);
        this.form.get('username')?.setValue(username);
        this.form.get('password')?.setValue(password);
      });
  }

  public async update(): Promise<void> {
    if (this.form.valid && this.form.dirty) {
      this.form.disable();

      const hostname = this.form.get('hostname')?.value;
      const topic = this.form.get('topic')?.value;
      const username = this.form.get('username')?.value;
      const password = this.form.get('password')?.value;

      await this.mqttService.saveMqttSettings(
        hostname,
        topic,
        username,
        password,
      );

      this.form.markAsPristine();
      this.form.enable();
    }
  }

  public async clear(): Promise<void> {
    this.form.disable();

    await this.mqttService.resetMqttSettings();

    this.form.reset();
    this.form.enable();
  }
}
