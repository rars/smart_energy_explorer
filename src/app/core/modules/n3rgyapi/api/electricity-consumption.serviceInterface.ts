/**
 * N3rgy.Data.Api
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */
import { HttpHeaders }                                       from '@angular/common/http';

import { Observable }                                        from 'rxjs';

import { ElectricityConsumptionDto } from '../model/models';


import { Configuration }                                     from '../configuration';



export interface ElectricityConsumptionServiceInterface {
    defaultHeaders: HttpHeaders;
    configuration: Configuration;

    /**
     * 
     * 
     * @param startDate 
     * @param endDate 
     */
    getElectricityConsumption(startDate?: string, endDate?: string, extraHttpRequestParams?: any): Observable<Array<ElectricityConsumptionDto>>;

}
