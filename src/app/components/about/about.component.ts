import { Component } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

@Component({
  selector: 'app-about',
  standalone: true,
  imports: [],
  templateUrl: './about.component.html',
  styleUrl: './about.component.scss',
})
export class AboutComponent {
  protected version: string = '';

  public constructor() {
    invoke<string>('get_app_version', {}).then((version) => {
      this.version = version;
    });
  }
}
