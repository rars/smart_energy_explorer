import { AbstractControl, ValidatorFn } from '@angular/forms';

export const exactLengthValidator = (requiredLength: number): ValidatorFn => {
  return (control: AbstractControl): { [key: string]: any } | null => {
    const length = control.value ? control.value.length : 0;
    if (length !== requiredLength) {
      return { exactLength: { required: requiredLength, actual: length } };
    }
    return null;
  };
};

export const noHyphenValidator = (): ValidatorFn => {
  return (control: AbstractControl): { [key: string]: any } | null => {
    const forbidden = /-/.test(control.value);
    return forbidden ? { noHyphen: { value: control.value } } : null;
  };
};
