export interface ElectricityMeterMessage {
  electricitymeter: ElectricityMeter;
}

interface ElectricityMeter {
  timestamp: string;
  energy: ElectricityEnergyContainer;
  power: Power;
}

interface ElectricityEnergyContainer {
  export: EnergyExport;
  import: EnergyImport;
}

interface EnergyExport {
  cumulative: number;
  units: string;
}

interface EnergyImport {
  cumulative: number;
  day: number;
  week: number;
  month: number;
  units: string;
  mpan: string;
  supplier: string;
  price: ImportPrice;
}

interface ImportPrice {
  unitrate: number;
  standingcharge: number;
}

interface Power {
  value: number;
  units: string;
}

export interface GasMeterMessage {
  gasmeter: GasMeter;
}

interface GasMeter {
  timestamp: string;
  energy: GasEnergyContainer;
}

interface GasEnergyContainer {
  import: GasImport;
}

interface GasImport {
  cumulative: number;
  day: number;
  week: number;
  month: number;
  units: string;

  cumulativevol: number;
  cumulativevolunits: string;

  dayvol: number;
  weekvol: number;
  monthvol: number;
  dayweekmonthvolunits: string;

  mprn: string;
  supplier: string;
  price: ImportPrice;
}
