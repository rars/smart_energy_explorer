export * from './electricity.service';
import { ElectricityService } from './electricity.service';
export * from './electricity.serviceInterface';
export * from './gas.service';
import { GasService } from './gas.service';
export * from './gas.serviceInterface';
export const APIS = [ElectricityService, GasService];
