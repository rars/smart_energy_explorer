import { Injectable } from '@angular/core';

import { Observable, from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';

import { ErrorService } from '../error/error.service';

type MqttSettings = {
  hostname: string;
  topic: string;
  username: string;
  password: string;
};

@Injectable({
  providedIn: 'root',
})
export class MqttService {
  public constructor(private readonly errorService: ErrorService) {}

  public getMqttSettings(): Observable<MqttSettings> {
    return from(invoke<MqttSettings>('get_mqtt_settings', {}));
  }

  public async saveMqttSettings(
    hostname: string,
    topic: string,
    username: string,
    password: string,
  ): Promise<void> {
    try {
      await invoke('store_mqtt_settings', {
        hostname,
        topic,
        username,
        password,
      });
    } catch (error) {
      this.errorService.showError(`Could not store MQTT settings: ${error}`);
      console.error(error);
    }
  }

  public async resetMqttSettings(): Promise<void> {
    try {
      await invoke('reset_mqtt_settings', {});
    } catch (error) {
      this.errorService.showError(`Could not reset MQTT settings: ${error}`);
      console.error(error);
    }
  }
}
