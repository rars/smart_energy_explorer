import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import {
  FormField,
  FormRoot,
  applyEach,
  form,
  required,
} from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterLink } from '@angular/router';

import { from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';

import { DateService } from '../../services/date/date.service';
import { CanComponentDeactivate } from '../../unsaved-changes.guard';

interface Profile {
  energyProfileId: number;
  name: string;
  startDate: Date;
  isActive: boolean;
}

interface ProfileSettings {
  profiles: Profile[];
}

@Component({
  selector: 'app-profile-settings',
  imports: [
    CommonModule,
    FormField,
    FormRoot,
    MatIconModule,
    MatButtonModule,
    MatCardModule,
    MatDatepickerModule,
    MatFormFieldModule,
    MatInputModule,
    MatSlideToggleModule,
    MatTooltipModule,
    RouterLink,
  ],
  templateUrl: './profile-settings.component.html',
  styleUrl: './profile-settings.component.scss',
})
export class ProfileSettingsComponent
  implements OnInit, CanComponentDeactivate
{
  public readonly data = signal<any[]>([]);
  public readonly profileSettings = signal<ProfileSettings>({ profiles: [] });
  protected readonly form = form(
    this.profileSettings,
    (path) => {
      applyEach(path.profiles, (profile) => {
        required(profile.name, { message: 'Profile name is required' });
      });
    },
    {
      submission: {
        action: async (f) => {
          const currentFormValues = f().value();
          const updateSuccessful = await this.update();
          if (updateSuccessful) {
            f().reset(currentFormValues);
          }
        },
      },
    },
  );

  public constructor(private readonly dateService: DateService) {}

  get profiles() {
    return this.form().value().profiles;
  }

  public ngOnInit(): void {
    from(invoke<any[]>('get_energy_profiles', {})).subscribe((x) => {
      this.data.set(x);

      const profiles = x.map((p) => ({
        energyProfileId: p.energyProfileId,
        name: p.name,
        startDate: new Date(p.startDate),
        isActive: p.isActive,
      }));

      this.profileSettings.set({ profiles });
    });
  }

  public async update(): Promise<boolean> {
    if (this.form().valid() && this.form().dirty()) {
      const energyProfileUpdates = this.form()
        .value()
        .profiles.map((x: any) => {
          return {
            energyProfileId: x.energyProfileId,
            startDate: this.dateService.formatISODate(x.startDate),
            isActive: x.isActive,
          };
        });

      await invoke('update_energy_profile_settings', {
        energyProfileUpdates,
      });

      return true;
    }

    return false;
  }

  public async canDeactivate(): Promise<boolean> {
    if (this.form().dirty()) {
      const confirmation = await confirm(
        'There are unsaved changes. Do you want to discard these changes?',
        { title: 'Discard changes?', kind: 'warning' },
      );
      return confirmation;
    }
    return true;
  }
}
