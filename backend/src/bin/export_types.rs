use specta::Types;
use specta_typescript::Typescript;
use std::path::Path;

use backend_lib::api::auth::*;
use backend_lib::api::calendars::*;
use backend_lib::api::contacts::*;
use backend_lib::api::messages::*;
use backend_lib::api::search::*;
use backend_lib::core::models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let types = Types::default()
        // Auth models
        .register::<RegisterRequest>()
        .register::<RegisterResponse>()
        .register::<TokenRequest>()
        .register::<TokenResponse>()
        .register::<LoginParams>()
        .register::<CallbackParams>()
        .register::<MeResponse>()
        // Core models
        .register::<Account>()
        .register::<Contact>()
        .register::<SettingsPayload>()
        .register::<LabelCustomization>()
        .register::<Snippet>()
        .register::<Signature>()
        // Messages models
        .register::<MessageListParams>()
        .register::<MessageListResponse>()
        .register::<MessageSummary>()
        .register::<MessageDetail>()
        .register::<StarParams>()
        .register::<LabelParams>()
        .register::<BulkActionType>()
        .register::<BulkActionParams>()
        .register::<SendAttachmentPayload>()
        .register::<SendMessageRequest>()
        .register::<SendMessageResponse>()
        // Calendar models
        .register::<CalendarSummary>()
        .register::<CalendarListResponse>()
        .register::<CalendarDetail>()
        .register::<EventListParams>()
        .register::<EventSummary>()
        .register::<EventListResponse>()
        .register::<EventDetail>()
        .register::<CreateEventRequest>()
        .register::<CreateEventResponse>()
        .register::<UpdateEventRequest>()
        // Search models
        .register::<SearchParams>()
        .register::<SearchResult>()
        .register::<SearchResponse>()
        .register::<SearchQuery>();

    let out_dir = Path::new("../frontend-shared/src/api/generated");
    std::fs::create_dir_all(out_dir)?;

    let out_file = out_dir.join("types.ts");
    let ts = Typescript::default();
    ts.export_to(&out_file, &types, &specta_serde::Format)?;

    println!("Successfully exported backend types to {}", out_file.display());
    Ok(())
}
