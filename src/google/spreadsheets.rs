use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::google::auth;
use once_cell::sync::OnceCell;
use reqwest::Client;
use yup_oauth2::authenticator::DefaultAuthenticator;

const SCOPES: &[&str] = &["https://www.googleapis.com/auth/spreadsheets"];

struct RestService {
    client: Client,
    auth: DefaultAuthenticator,
}

static SERVICE: OnceCell<RestService> = OnceCell::new();

pub(crate) async fn initialize<'a>(key: &str) {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap();
    let auth = auth(key, SCOPES).await;
    let inner = RestService { client, auth };
    if SERVICE.set(inner).is_err() {
        panic!(concat!("Already initialized sheets service"));
    }
}

pub struct Range {
    pub sheet: String,
    pub start: String,
    pub end: Option<String>,
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(end) = &self.end {
            write!(
                f,
                "'{}'!{}:{}",
                self.sheet,
                self.start.to_ascii_uppercase(),
                end.to_ascii_uppercase()
            )
        } else {
            write!(f, "'{}'!{}", self.sheet, self.start.to_ascii_uppercase(),)
        }
    }
}

/// Indicates which dimension an operation should apply to.
pub enum Dimension {
    /// Operates on the rows of a sheet.
    Rows,
    /// Operates on the columns of a sheet.
    Columns,
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Rows => write!(f, "ROWS"),
            Dimension::Columns => write!(f, "COLUMNS"),
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::Rows
    }
}

/// Determines how values should be rendered in the output.
pub enum ValueRenderOption {
    /// Values will be calculated & formatted in the reply according to the cell's formatting.
    /// Formatting is based on the spreadsheet's locale, not the requesting user's locale.
    /// For example, if A1 is 1.23 and A2 is =A1 and formatted as currency, then A2 would return "$1.23".
    FormattedValue,
    /// Values will be calculated, but not formatted in the reply.
    /// For example, if A1 is 1.23 and A2 is =A1 and formatted as currency, then A2 would return the number 1.23.
    UnformattedValue,
    /// Values will not be calculated. The reply will include the formulas.
    /// For example, if A1 is 1.23 and A2 is =A1 and formatted as currency, then A2 would return "=A1".
    Formula,
}

impl Default for ValueRenderOption {
    fn default() -> Self {
        ValueRenderOption::FormattedValue
    }
}

impl Display for ValueRenderOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueRenderOption::FormattedValue => write!(f, "FORMATTED_VALUE"),
            ValueRenderOption::UnformattedValue => write!(f, "UNFORMATTED_VALUE"),
            ValueRenderOption::Formula => write!(f, "FORMULA"),
        }
    }
}

impl Serialize for ValueRenderOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Determines how dates should be rendered in the output.
pub enum DateTimeRenderOption {
    /// Instructs date, time, datetime, and duration fields to be output as
    /// doubles in "serial number" format, as popularized by Lotus 1-2-3.
    /// The whole number portion of the value (left of the decimal) counts the
    /// days since December 30th 1899.
    /// The fractional portion (right of the decimal) counts the time as a fraction of the day.
    /// For example, January 1st 1900 at noon would be 2.5,
    /// 2 because it's 2 days after December 30st 1899, and .5 because noon is half a day.
    /// February 1st 1900 at 3pm would be 33.625. This correctly treats the year 1900 as not a leap year.
    SerialNumber,
    /// Instructs date, time, datetime, and duration fields to be output as strings
    /// in their given number format (which is dependent on the spreadsheet locale).
    FormattedString,
}

impl Display for DateTimeRenderOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateTimeRenderOption::SerialNumber => write!(f, "SERIAL_NUMBER"),
            DateTimeRenderOption::FormattedString => write!(f, "FORMATTED_STRING"),
        }
    }
}

impl Default for DateTimeRenderOption {
    fn default() -> Self {
        DateTimeRenderOption::FormattedString
    }
}

impl Serialize for DateTimeRenderOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Determines how input data should be interpreted.
pub enum ValueInputOption {
    /// The values the user has entered will not be parsed and will be stored as-is.
    Raw,
    /// The values will be parsed as if the user typed them into the UI.
    /// Numbers will stay as numbers, but strings may be converted to numbers,
    /// dates, etc. following the same rules that are applied when entering text
    /// into a cell via the Google Sheets UI.
    UserEntered,
}

impl Display for ValueInputOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueInputOption::Raw => write!(f, "RAW"),
            ValueInputOption::UserEntered => write!(f, "USER_ENTERED"),
        }
    }
}

impl Default for ValueInputOption {
    fn default() -> Self {
        ValueInputOption::UserEntered
    }
}

impl Serialize for ValueInputOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Determines how existing data is changed when new data is input.
pub enum InsertDataOption {
    /// The new data overwrites existing data in the areas it is written.
    /// (Note: adding data to the end of the sheet will still insert new rows
    /// or columns so the data can be written.)
    Overwrite,
    /// Rows are inserted for the new data.
    InsertRows,
}

impl Display for InsertDataOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsertDataOption::Overwrite => write!(f, "OVERWRITE"),
            InsertDataOption::InsertRows => write!(f, "INSERT_ROWS"),
        }
    }
}

impl Default for InsertDataOption {
    fn default() -> Self {
        InsertDataOption::Overwrite
    }
}

/// Data within a range of the spreadsheet.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [values append spreadsheets](struct.SpreadsheetValueAppendCall.html) (request)
/// * [values get spreadsheets](struct.SpreadsheetValueGetCall.html) (response)
/// * [values update spreadsheets](struct.SpreadsheetValueUpdateCall.html) (request)
///
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRange {
    /// The range the values cover, in A1 notation.
    /// For output, this range indicates the entire requested range,
    /// even though the values will exclude trailing rows and columns.
    /// When appending values, this field represents the range to search for a
    /// table, after which values will be appended.
    pub range: Option<String>,
    /// The data that was read or to be written.  This is an array of arrays,
    /// the outer array representing all the data and each inner array
    /// representing a major dimension. Each item in the inner array
    /// corresponds with one cell.
    ///
    /// For output, empty trailing rows and columns will not be included.
    ///
    /// For input, supported value types are: bool, string, and double.
    /// Null values will be skipped.
    /// To set a cell to an empty value, set the string value to an empty string.
    pub values: Option<Vec<Vec<Option<String>>>>,
    /// The major dimension of the values.
    ///
    /// For output, if the spreadsheet data is: `A1=1,B1=2,A2=3,B2=4`,
    /// then requesting `range=A1:B2,majorDimension=ROWS` will return
    /// `[[1,2],[3,4]]`,
    /// whereas requesting `range=A1:B2,majorDimension=COLUMNS` will return
    /// `[[1,3],[2,4]]`.
    ///
    /// For input, with `range=A1:B2,majorDimension=ROWS` then `[[1,2],[3,4]]`
    /// will set `A1=1,B1=2,A2=3,B2=4`. With `range=A1:B2,majorDimension=COLUMNS`
    /// then `[[1,2],[3,4]]` will set `A1=1,B1=3,A2=2,B2=4`.
    ///
    /// When writing, if this field is not set, it defaults to ROWS.
    pub major_dimension: Option<String>,
}

pub struct GetParams<'a> {
    /// The ID of the spreadsheet to retrieve data from.
    pub spreadsheet_id: &'a str,
    /// The A1 notation of the values to retrieve.
    pub range: Range,
    /// The major dimension that results should use.
    ///
    /// For example, if the spreadsheet data is: A1=1,B1=2,A2=3,B2=4,
    /// then requesting range=A1:B2,majorDimension=ROWS returns [[1,2],[3,4]],
    /// whereas requesting range=A1:B2,majorDimension=COLUMNS returns [[1,3],[2,4]].
    ///
    /// The default dimension is Dimension::Rows.
    pub major_dimension: Option<Dimension>,
    /// How values should be represented in the output.
    /// The default render option is ValueRenderOption::FormattedValue.
    pub value_render_option: Option<ValueRenderOption>,
    /// How dates, times, and durations should be represented in the output.
    /// This is ignored if valueRenderOption is FORMATTED_VALUE.
    /// The default dateTime render option is [DateTimeRenderOption::FormattedString].
    pub date_time_render_option: Option<DateTimeRenderOption>,
}

/// Returns a range of values from a spreadsheet. The caller must specify the spreadsheet ID and a range.
pub async fn get<'a>(params: GetParams<'_>) -> Result<ValueRange, String> {
    // GET https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values/{range}
    let mut query_params = Vec::with_capacity(6);

    query_params.push((
        "majorDimension",
        params.major_dimension.unwrap_or_default().to_string(),
    ));
    query_params.push((
        "valueRenderOption",
        params.value_render_option.unwrap_or_default().to_string(),
    ));
    query_params.push((
        "dateTimeRenderOption",
        params
            .date_time_render_option
            .unwrap_or_default()
            .to_string(),
    ));
    query_params.push(("alt", "json".to_string()));

    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        params.spreadsheet_id,
        params.range.to_string()
    );
    let url = reqwest::Url::parse_with_params(&url, &query_params).unwrap();

    let service = SERVICE.get().unwrap();
    let token = service.auth.token(SCOPES).await.unwrap();

    let result = service
        .client
        .get(url)
        .bearer_auth(token.as_str())
        .send()
        .await;

    match result {
        Ok(result) => {
            if !result.status().is_success() {
                return Err(format!(
                    "Unexpected result with status code {}: {}",
                    result.status(),
                    result.text().await.unwrap()
                ));
            }
            let result = result.json().await;
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub struct UpdateParams<'a> {
    /// The ID of the spreadsheet to update.
    pub spreadsheet_id: &'a str,
    /// The A1 notation of the values to update.
    pub range: Range,
    /// Data to upload.
    pub values: ValueRange,
    /// How the input data should be interpreted.
    pub value_input_option: Option<ValueInputOption>,
    /// Determines if the update response should include the values of the cells that were updated.
    /// By default, responses do not include the updated values.
    /// If the range to write was larger than the range actually written,
    /// the response includes all values in the requested range (excluding trailing empty rows and columns).
    pub include_values_in_response: Option<bool>,
    /// Determines how values in the response should be rendered.
    /// The default render option is ValueRenderOption::FormattedValue.
    pub response_value_render_option: Option<ValueRenderOption>,
    /// Determines how dates, times, and durations in the response should be rendered.
    /// This is ignored if responseValueRenderOption is FORMATTED_VALUE.
    /// The default dateTime render option is DateTimeRenderOption::SerialNumber.
    pub response_date_time_render_option: Option<DateTimeRenderOption>,
}

/// The response when updating a range of values in a spreadsheet.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [values update spreadsheets](struct.SpreadsheetValueUpdateCall.html) (response)
///
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateValuesResponse {
    /// The number of columns where at least one cell in the column was updated.
    pub updated_columns: Option<i32>,
    /// The range (in A1 notation) that updates were applied to.
    pub updated_range: Option<String>,
    /// The number of rows where at least one cell in the row was updated.
    pub updated_rows: Option<i32>,
    /// The values of the cells after updates were applied.
    /// This is only included if the request's `includeValuesInResponse` field
    /// was `true`.
    pub updated_data: Option<ValueRange>,
    /// The spreadsheet the updates were applied to.
    pub spreadsheet_id: Option<String>,
    /// The number of cells updated.
    pub updated_cells: Option<i32>,
}

/// Sets values in a range of a spreadsheet. The caller must specify the
/// spreadsheet ID, range, and a valueInputOption.
pub async fn update<'a>(params: UpdateParams<'_>) -> Result<UpdateValuesResponse, String> {
    // PUT https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values/{range}
    let mut query_params = Vec::with_capacity(6);

    query_params.push((
        "valueInputOption",
        params.value_input_option.unwrap_or_default().to_string(),
    ));
    query_params.push((
        "includeValuesInResponse",
        params
            .include_values_in_response
            .unwrap_or_default()
            .to_string(),
    ));
    query_params.push((
        "responseDateTimeRenderOption",
        params
            .response_date_time_render_option
            .unwrap_or_default()
            .to_string(),
    ));
    query_params.push((
        "responseValueRenderOption",
        params
            .response_value_render_option
            .unwrap_or_default()
            .to_string(),
    ));
    query_params.push(("alt", "json".to_string()));

    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        params.spreadsheet_id,
        params.range.to_string()
    );

    let url = reqwest::Url::parse_with_params(&url, &query_params).unwrap();

    let service = SERVICE.get().unwrap();
    let token = service.auth.token(SCOPES).await.unwrap();

    let result = service
        .client
        .put(url)
        .json(&params.values)
        .bearer_auth(token.as_str())
        .send()
        .await;

    match result {
        Ok(result) => {
            if !result.status().is_success() {
                return Err(format!(
                    "Unexpected result with status code {}: {}",
                    result.status(),
                    result.text().await.unwrap()
                ));
            }
            let result = result.json().await;
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub struct AppendParams<'a> {
    /// The ID of the spreadsheet to update.
    pub spreadsheet_id: &'a str,
    /// The A1 notation of a range to search for a logical table of data.
    /// Values are appended after the last row of the table.
    pub range: Range,
    /// Values to append.
    pub values: ValueRange,
    /// How the input data should be interpreted.
    pub value_input_option: Option<ValueInputOption>,
    /// How the input data should be inserted.
    pub insert_data_option: Option<InsertDataOption>,
    /// Determines if the update response should include the values of the cells that were appended.
    /// By default, responses do not include the updated values.
    pub include_values_in_response: Option<bool>,
    /// Determines how values in the response should be rendered.
    /// The default render option is ValueRenderOption.FORMATTED_VALUE.
    pub response_value_render_option: Option<ValueRenderOption>,
    /// Determines how dates, times, and durations in the response should be rendered.
    /// This is ignored if responseValueRenderOption is FORMATTED_VALUE.
    /// The default dateTime render option is [DateTimeRenderOption::FormattedString].
    pub response_date_time_render_option: Option<DateTimeRenderOption>,
}

/// The response when updating a range of values in a spreadsheet.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [values append spreadsheets](struct.SpreadsheetValueAppendCall.html) (response)
///
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendValuesResponse {
    /// The spreadsheet the updates were applied to.
    pub spreadsheet_id: Option<String>,
    /// The range (in A1 notation) of the table that values are being appended to
    /// (before the values were appended).
    /// Empty if no table was found.
    pub table_range: Option<String>,
    /// Information about the updates that were applied.
    pub updates: Option<UpdateValuesResponse>,
}

/// Appends values to a spreadsheet. The input range is used to search for existing data
/// and find a "table" within that range. Values will be appended to the next
/// row of the table, starting with the first column of the table.
/// See the guide and sample code for specific details of how tables are detected and data is appended.
///
/// The caller must specify the spreadsheet ID, range,
/// and a valueInputOption. The valueInputOption only controls
/// how the input data will be added to the sheet (column-wise or row-wise),
/// it does not influence what cell the data starts being written to.
pub async fn append<'a>(params: AppendParams<'_>) -> Result<AppendValuesResponse, String> {
    // POST https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values/{range}:append
    let query_params = vec![
        (
            "valueInputOption",
            params.value_input_option.unwrap_or_default().to_string(),
        ),
        (
            "includeValuesInResponse",
            params
                .include_values_in_response
                .unwrap_or_default()
                .to_string(),
        ),
        (
            "insertDataOption",
            params.insert_data_option.unwrap_or_default().to_string(),
        ),
        (
            "responseDateTimeRenderOption",
            params
                .response_date_time_render_option
                .unwrap_or_default()
                .to_string(),
        ),
        (
            "responseValueRenderOption",
            params
                .response_value_render_option
                .unwrap_or_default()
                .to_string(),
        ),
        ("alt", "json".to_string()),
    ];

    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:append",
        params.spreadsheet_id,
        params.range.to_string()
    );

    let url = reqwest::Url::parse_with_params(&url, &query_params).unwrap();

    let service = SERVICE.get().unwrap();
    let token = service.auth.token(SCOPES).await.unwrap();

    let result = service
        .client
        .post(url)
        .json(&params.values)
        .bearer_auth(token.as_str())
        .send()
        .await;

    match result {
        Ok(result) => {
            if !result.status().is_success() {
                return Err(format!(
                    "Unexpected result with status code {}: {}",
                    result.status(),
                    result.text().await.unwrap()
                ));
            }
            let result = result.json().await;
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub struct BatchGetParams<'a> {
    /// The ID of the spreadsheet to retrieve data from.
    pub spreadsheet_id: &'a str,
    /// The A1 notation of the values to retrieve.
    pub ranges: Vec<Range>,
    /// The major dimension that results should use.
    ///
    /// For example, if the spreadsheet data is: A1=1,B1=2,A2=3,B2=4,
    /// then requesting range=A1:B2,majorDimension=ROWS returns [[1,2],[3,4]],
    /// whereas requesting range=A1:B2,majorDimension=COLUMNS returns [[1,3],[2,4]].
    pub major_dimension: Option<Dimension>,
    /// How values should be represented in the output.
    /// The default render option is ValueRenderOption.FORMATTED_VALUE.
    pub value_render_option: Option<ValueRenderOption>,
    /// How dates, times, and durations should be represented in the output. This is ignored if valueRenderOption is FORMATTED_VALUE.
    /// The default dateTime render option is [DateTimeRenderOption::FormattedString].
    pub date_time_render_option: Option<DateTimeRenderOption>,
}

/// The response when retrieving more than one range of values in a spreadsheet.
///
/// # Activities
///
/// This type is used in activities, which are methods you may call on this type or where this type is involved in.
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
///
/// * [values batch get spreadsheets](struct.SpreadsheetValueBatchGetCall.html) (response)
///
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchGetValuesResponse {
    /// The ID of the spreadsheet the data was retrieved from.
    pub spreadsheet_id: Option<String>,
    /// The requested values. The order of the ValueRanges is the same as the
    /// order of the requested ranges.
    pub value_ranges: Option<Vec<ValueRange>>,
}

/// Returns one or more ranges of values from a spreadsheet.
/// The caller must specify the spreadsheet ID and one or more ranges.
pub async fn batch_get<'a>(params: BatchGetParams<'_>) -> Result<BatchGetValuesResponse, String> {
    // GET https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values:batchGet
    let mut query_params = Vec::with_capacity(4 + params.ranges.len());

    for range in params.ranges {
        query_params.push(("ranges", range.to_string()));
    }

    query_params.push((
        "majorDimension",
        params.major_dimension.unwrap_or_default().to_string(),
    ));
    query_params.push((
        "dateTimeRenderOption",
        params
            .date_time_render_option
            .unwrap_or_default()
            .to_string(),
    ));
    query_params.push((
        "valueRenderOption",
        params.value_render_option.unwrap_or_default().to_string(),
    ));
    query_params.push(("alt", "json".to_string()));

    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values:batchGet",
        params.spreadsheet_id
    );
    let url = reqwest::Url::parse_with_params(&url, &query_params).unwrap();

    let service = SERVICE.get().unwrap();
    let token = service.auth.token(SCOPES).await.unwrap();

    let result = service
        .client
        .get(url)
        .bearer_auth(token.as_str())
        .send()
        .await;

    match result {
        Ok(result) => {
            if !result.status().is_success() {
                return Err(format!(
                    "Unexpected result with status code {}: {}",
                    result.status(),
                    result.text().await.unwrap()
                ));
            }
            let result = result.json().await;
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Deserialize)]
pub struct SheetProperties {
    pub title: String,
    // there are lot more but i skipped rest of fields
    // https://developers.google.com/sheets/api/reference/rest/v4/spreadsheets/sheets#SheetProperties
}

#[derive(Deserialize)]
pub struct Sheet {
    pub properties: SheetProperties,
    // there are lot more but i skipped rest of fields
    // https://developers.google.com/sheets/api/reference/rest/v4/spreadsheets/sheets#Sheet
}

#[derive(Deserialize)]
pub struct Spreadsheet {
    pub sheets: Vec<Sheet>,
    // there are lot more but i skipped rest of fields
    // https://developers.google.com/sheets/api/reference/rest/v4/spreadsheets#Spreadsheet
}

pub async fn get_spreadsheet_info(spreadsheet_id: &str) -> Result<Spreadsheet, String> {
    // GET https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}",
        spreadsheet_id
    );
    let url = reqwest::Url::parse(&url).unwrap();

    let service = SERVICE.get().unwrap();
    let token = service.auth.token(SCOPES).await.unwrap();

    let result = service
        .client
        .get(url)
        .bearer_auth(token.as_str())
        .send()
        .await;

    match result {
        Ok(result) => {
            if !result.status().is_success() {
                return Err(format!(
                    "Unexpected result with status code {}: {}",
                    result.status(),
                    result.text().await.unwrap()
                ));
            }
            let result = result.json().await;
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateParams<'a> {
    /// The ID of the spreadsheet to update.
    pub spreadsheet_id: &'a str,
    /// How the input data should be interpreted.
    pub value_input_option: Option<ValueInputOption>,
    /// The new values to apply to the spreadsheet.
    pub data: Vec<ValueRange>,
    /// Determines if the update response should include the values of the cells that were updated.
    /// By default, responses do not include the updated values.
    /// The updatedData field within each of the BatchUpdateValuesResponse.responses
    /// contains the updated values. If the range to write was larger than the range actually written,
    /// the response includes all values in the requested range (excluding trailing empty rows and columns).
    pub include_values_in_response: Option<bool>,
    /// Determines how values in the response should be rendered.
    /// The default render option is ValueRenderOption.FORMATTED_VALUE.
    pub response_value_render_option: Option<ValueRenderOption>,
    /// Determines how dates, times, and durations in the response should be rendered.
    /// This is ignored if responseValueRenderOption is FORMATTED_VALUE.
    /// The default dateTime render option is DateTimeRenderOption.SERIAL_NUMBER.
    pub response_date_time_render_option: Option<DateTimeRenderOption>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub spreadsheet_id: Option<String>,
    pub total_updated_rows: Option<usize>,
    pub total_updated_columns: Option<usize>,
    pub total_updated_cells: Option<usize>,
    pub total_updated_sheets: Option<usize>,
}

/// Sets values in one or more ranges of a spreadsheet.
/// The caller must specify the spreadsheet ID, a valueInputOption, and one or more ValueRanges.
pub async fn batch_update<'a>(
    params: BatchUpdateParams<'_>,
) -> Result<BatchUpdateResponse, String> {
    // POST https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values:batchUpdate
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values:batchUpdate",
        params.spreadsheet_id
    );
    let url = reqwest::Url::parse(&url).unwrap();

    let service = SERVICE.get().unwrap();
    let token = service.auth.token(SCOPES).await.unwrap();

    let result = service
        .client
        .post(url)
        .json(&params)
        .bearer_auth(token.as_str())
        .send()
        .await;

    match result {
        Ok(result) => {
            if !result.status().is_success() {
                return Err(format!(
                    "Unexpected result with status code {}: {}",
                    result.status(),
                    result.text().await.unwrap()
                ));
            }
            let result = result.json().await;
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
