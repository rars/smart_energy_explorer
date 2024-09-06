import { Component, HostListener, OnInit } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { ApiModule } from './core/modules/n3rgyapi';
import { CommonModule } from '@angular/common';
import { NavigationBarComponent } from './components/navigation-bar/navigation-bar.component';
import { MatIconRegistry } from '@angular/material/icon';

import Database from '@tauri-apps/plugin-sql';
import { DataDownloadingComponent } from './components/data-downloading/data-downloading.component';
import { StatusBarComponent } from './status-bar/status-bar.component';
// when using `"withGlobalTauri": true`, you may use
// const V = window.__TAURI_PLUGIN_SQL__;

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    RouterOutlet,
    ApiModule,
    CommonModule,
    NavigationBarComponent,
    DataDownloadingComponent,
    StatusBarComponent,
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent implements OnInit {
  public width = 800;
  public height = 300;

  public data: any;

  public constructor(private readonly matIconRegistry: MatIconRegistry) {
    this.matIconRegistry.setDefaultFontSetClass('material-symbols-outlined');
  }

  public ngOnInit(): void {
    // this.fetchData();
    // this.rustClient();
    //this.resizeWindow();
  }

  @HostListener('window:resize', ['$event'])
  public onResize(event: Event) {
    if (event.target) {
      const target = event.target as unknown as Window;
      this.width = target.innerWidth;
      this.height = target.innerHeight;
    }
  }

  /*resizeWindow() {
    const appWindow = Window.getCurrent();
    const { width, height } = this.getContentSize();
    appWindow.setSize(new LogicalSize(width, height));
  }

  getContentSize() {
    const content = document.getElementById('main-content');
    return {
      width: content?.offsetWidth ?? 800,
      height: content?.offsetHeight ?? 400,
    };
  }*/

  async fetchData() {
    try {
      const db = await Database.load('sqlite:./my_n3rgy.db');
      const data = await db.select(
        'SELECT timestamp, energy_consumption_kwh FROM electricity_consumption;',
        //`SELECT * FROM sqlite_master WHERE type='table';`
      );
      this.data = data;
    } catch (e) {}
  }
}
