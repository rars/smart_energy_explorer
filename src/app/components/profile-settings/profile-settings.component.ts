import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import {
  FormArray,
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
} from '@angular/forms';
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

@Component({
  selector: 'app-profile-settings',
  imports: [
    CommonModule,
    MatIconModule,
    MatButtonModule,
    MatCardModule,
    MatDatepickerModule,
    MatFormFieldModule,
    MatInputModule,
    MatSlideToggleModule,
    MatTooltipModule,
    ReactiveFormsModule,
    RouterLink,
  ],
  templateUrl: './profile-settings.component.html',
  styleUrl: './profile-settings.component.scss',
})
export class ProfileSettingsComponent
  implements OnInit, CanComponentDeactivate
{
  public form: FormGroup;
  public data?: any;

  public constructor(
    private readonly dateService: DateService,
    private readonly fb: FormBuilder,
  ) {
    this.form = this.fb.group({
      profiles: this.fb.array([]),
    });
  }

  get profiles() {
    return this.form.get('profiles') as FormArray;
  }

  public ngOnInit(): void {
    from(invoke<any[]>('get_energy_profiles', {})).subscribe((x) => {
      this.data = x;

      for (const p of x) {
        this.profiles.push(
          this.fb.group({
            energyProfileId: p.energyProfileId,
            name: p.name,
            startDate: new Date(p.startDate),
            isActive: p.isActive,
          }),
        );
      }
    });
  }

  public update(): void {
    if (this.form.valid && this.form.dirty) {
      this.form.disable();

      const energyProfileUpdates = this.form.value.profiles.map((x: any) => {
        return {
          energyProfileId: x.energyProfileId,
          startDate: this.dateService.formatISODate(x.startDate),
          isActive: x.isActive,
        };
      });

      from(
        invoke('update_energy_profile_settings', {
          energyProfileUpdates,
        }),
      ).subscribe(() => {
        this.form.markAsPristine();
        this.form.enable();
      });
    }
  }

  public async canDeactivate(): Promise<boolean> {
    if (this.form.dirty) {
      const confirmation = await confirm(
        'There are unsaved changes. Do you want to discard these changes?',
        { title: 'Discard changes?', kind: 'warning' },
      );
      debugger;
      return confirmation;
    }
    return true;
  }
}
