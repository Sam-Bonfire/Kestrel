use specta::Types;
use specta_typescript::Typescript;
use std::path::Path;

use backend::api::auth::*;
use backend::api::availability::*;
use backend::api::booking::*;
use backend::api::calendars::*;
use backend::api::clips::*;
use backend::api::contacts::*;
use backend::api::messages::*;
use backend::api::polls::*;
use backend::api::search::*;
use backend::api::threads::*;
use backend::api::vacation::*;
use backend::core::models::*;

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
        .register::<NotificationPrefs>()
        .register::<AiCapabilityToggles>()
        // Messages models
        .register::<MessageListParams>()
        .register::<MessageListResponse>()
        .register::<MessageSummary>()
        .register::<MessageDetail>()
        .register::<StarParams>()
        .register::<LabelParams>()
        .register::<Clip>()
        .register::<CreateClipRequest>()
        .register::<ThreadNote>()
        .register::<SetNoteRequest>()
        .register::<SetAsideParams>()
        .register::<ThreadSubjectOverride>()
        .register::<SetSubjectRequest>()
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
        .register::<SearchQuery>()
        // Vacation models
        .register::<VacationDto>()
        .register::<VacationUpdate>()
        // Event poll models
        .register::<EventPoll>()
        .register::<EventPollWithVotes>()
        .register::<PollVote>()
        .register::<CreatePollRequest>()
        .register::<VoteRequest>()
        // Availability models
        .register::<FreebusyRequest>()
        .register::<BusyBlockDto>()
        // Booking page models
        .register::<BookingPage>()
        .register::<CreateBookingPageRequest>()
        .register::<UpdateBookingPageRequest>()
        .register::<PublicBookingDto>()
        .register::<SlotDto>()
        .register::<SlotsResponse>()
        .register::<BookSlotRequest>()
        .register::<BookSlotResponse>();

    let out_dir = Path::new("../frontend-shared/src/api/generated");
    std::fs::create_dir_all(out_dir)?;

    let out_file = out_dir.join("types.ts");
    let ts = Typescript::default();
    ts.export_to(&out_file, &types, &specta_serde::Format)?;

    println!(
        "Successfully exported backend types to {}",
        out_file.display()
    );
    Ok(())
}
