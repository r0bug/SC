mod auth;
mod commands;
mod commands_extended;
mod import;
mod import_large;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sagenscontact")]
#[command(about = "SagensContact - Portable Contact Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Authentication commands
    Login {
        email: String,
        password: String,
    },
    Signup {
        email: String,
        password: String,
        name: String,
    },
    Logout,
    Status,
    Import {
        #[arg(short, long)]
        csv: Option<String>,
        #[arg(short, long)]
        vcard: Option<String>,
        #[arg(short, long)]
        sms: Option<String>,
        #[arg(long)]
        google_csv: Option<String>,
        #[arg(long)]
        sms_xml_large: Option<String>,
        #[arg(long)]
        calls_xml: Option<String>,
    },
    List {
        #[arg(short, long, default_value = "50")]
        limit: i64,
    },
    Search {
        query: String,
    },
    Add {
        first_name: String,
        last_name: Option<String>,
        #[arg(short, long)]
        email: Option<String>,
        #[arg(short, long)]
        phone: Option<String>,
    },
    Note {
        contact_id: String,
        title: String,
        content: String,
    },
    Communicate {
        contact_id: String,
        method: String,
        message: String,
    },
    Share {
        entity_type: String,
        entity_id: String,
        email: String,
    },
    Suggest {
        contact_id: String,
    },
    // Group commands
    GroupCreate {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },
    GroupList {
        #[arg(short, long, default_value = "50")]
        limit: i64,
    },
    GroupAddMember {
        group_id: String,
        contact_id: String,
    },
    GroupRemoveMember {
        group_id: String,
        contact_id: String,
    },
    GroupDelete {
        group_id: String,
    },
    // Concept commands
    ConceptCreate {
        name: String,
        content: String,
        #[arg(short, long)]
        category: Option<String>,
    },
    ConceptList {
        #[arg(short, long, default_value = "50")]
        limit: i64,
    },
    ConceptSearch {
        query: String,
    },
    ConceptDelete {
        concept_id: String,
    },
    // Calendar event commands
    EventCreate {
        title: String,
        start_time: String,
        end_time: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        location: Option<String>,
        #[arg(short, long)]
        contact_ids: Vec<String>,
    },
    EventList {
        #[arg(short, long)]
        from_date: Option<String>,
        #[arg(short, long)]
        to_date: Option<String>,
    },
    EventUpdate {
        event_id: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        start_time: Option<String>,
        #[arg(short, long)]
        end_time: Option<String>,
    },
    EventDelete {
        event_id: String,
    },
    // Attachment commands
    AttachmentUpload {
        file_path: String,
        entity_type: String,
        entity_id: String,
        #[arg(short, long)]
        description: Option<String>,
    },
    AttachmentDownload {
        attachment_id: String,
        #[arg(short, long)]
        output_path: Option<String>,
    },
    AttachmentList {
        entity_type: String,
        entity_id: String,
    },
    AttachmentDelete {
        attachment_id: String,
    },
    // Share management commands
    ShareList {
        #[arg(short, long)]
        entity_type: Option<String>,
    },
    ShareAccept {
        invite_id: String,
    },
    ShareRevoke {
        invite_id: String,
    },
    ShareUpdate {
        invite_id: String,
        permissions: Vec<String>,
    },
    // Search history commands
    HistoryList {
        #[arg(short, long, default_value = "50")]
        limit: i64,
    },
    HistoryClear,
    HistoryDelete {
        history_id: String,
    },
    // AI insight commands
    InsightReview {
        entity_type: String,
        entity_id: String,
    },
    InsightApply {
        insight_id: String,
    },
    InsightFeedback {
        insight_id: String,
        rating: i32,
        #[arg(short, long)]
        comment: Option<String>,
    },
    InsightList {
        #[arg(short, long)]
        entity_type: Option<String>,
        #[arg(short, long, default_value = "50")]
        limit: i64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        // Authentication
        Commands::Login { email, password } => {
            commands::login_command(email, password).await?;
        }
        Commands::Signup {
            email,
            password,
            name,
        } => {
            commands::signup_command(email, password, name).await?;
        }
        Commands::Logout => {
            commands::logout_command().await?;
        }
        Commands::Status => {
            commands::status_command().await?;
        }
        // Original commands
        Commands::Import {
            csv,
            vcard,
            sms,
            google_csv,
            sms_xml_large,
            calls_xml,
        } => {
            // Handle new streaming importers
            if let Some(path) = google_csv {
                let store = local_store::LocalStore::new("sqlite:./data/contacts.db").await?;
                import_large::import_google_contacts_csv(&path, &store).await?;
            } else if let Some(path) = sms_xml_large {
                let store = local_store::LocalStore::new("sqlite:./data/contacts.db").await?;
                import_large::import_sms_xml_streaming(&path, &store).await?;
            } else if let Some(path) = calls_xml {
                let store = local_store::LocalStore::new("sqlite:./data/contacts.db").await?;
                import_large::import_calls_xml_streaming(&path, &store).await?;
            } else {
                // Fall back to original import command
                commands::import_command(csv, vcard, sms).await?;
            }
        }
        Commands::List { limit } => {
            commands::list_command(limit).await?;
        }
        Commands::Search { query } => {
            commands::search_command(&query).await?;
        }
        Commands::Add {
            first_name,
            last_name,
            email,
            phone,
        } => {
            commands::add_command(first_name, last_name, email, phone).await?;
        }
        Commands::Note {
            contact_id,
            title,
            content,
        } => {
            commands::note_command(&contact_id, title, content).await?;
        }
        Commands::Communicate {
            contact_id,
            method,
            message,
        } => {
            commands::communicate_command(&contact_id, &method, message).await?;
        }
        Commands::Share {
            entity_type,
            entity_id,
            email,
        } => {
            commands::share_command(&entity_type, &entity_id, &email).await?;
        }
        Commands::Suggest { contact_id } => {
            commands::suggest_command(&contact_id).await?;
        }
        // Group commands
        Commands::GroupCreate { name, description } => {
            commands::group_create_command(name, description).await?;
        }
        Commands::GroupList { limit } => {
            commands::group_list_command(limit).await?;
        }
        Commands::GroupAddMember {
            group_id,
            contact_id,
        } => {
            commands::group_add_member_command(&group_id, &contact_id).await?;
        }
        Commands::GroupRemoveMember {
            group_id,
            contact_id,
        } => {
            commands::group_remove_member_command(&group_id, &contact_id).await?;
        }
        Commands::GroupDelete { group_id } => {
            commands::group_delete_command(&group_id).await?;
        }
        // Concept commands
        Commands::ConceptCreate {
            name,
            content,
            category,
        } => {
            commands::concept_create_command(name, content, category).await?;
        }
        Commands::ConceptList { limit } => {
            commands::concept_list_command(limit).await?;
        }
        Commands::ConceptSearch { query } => {
            commands::concept_search_command(&query).await?;
        }
        Commands::ConceptDelete { concept_id } => {
            commands::concept_delete_command(&concept_id).await?;
        }
        // Calendar event commands
        Commands::EventCreate {
            title,
            start_time,
            end_time,
            description,
            location,
            contact_ids,
        } => {
            commands::event_create_command(
                title,
                start_time,
                end_time,
                description,
                location,
                contact_ids,
            )
            .await?;
        }
        Commands::EventList { from_date, to_date } => {
            commands::event_list_command(from_date, to_date).await?;
        }
        Commands::EventUpdate {
            event_id,
            title,
            start_time,
            end_time,
        } => {
            commands::event_update_command(&event_id, title, start_time, end_time).await?;
        }
        Commands::EventDelete { event_id } => {
            commands::event_delete_command(&event_id).await?;
        }
        // Attachment commands
        Commands::AttachmentUpload {
            file_path,
            entity_type,
            entity_id,
            description,
        } => {
            commands::attachment_upload_command(&file_path, &entity_type, &entity_id, description)
                .await?;
        }
        Commands::AttachmentDownload {
            attachment_id,
            output_path,
        } => {
            commands::attachment_download_command(&attachment_id, output_path).await?;
        }
        Commands::AttachmentList {
            entity_type,
            entity_id,
        } => {
            commands::attachment_list_command(&entity_type, &entity_id).await?;
        }
        Commands::AttachmentDelete { attachment_id } => {
            commands::attachment_delete_command(&attachment_id).await?;
        }
        // Share management commands
        Commands::ShareList { entity_type } => {
            commands::share_list_command(entity_type).await?;
        }
        Commands::ShareAccept { invite_id } => {
            commands::share_accept_command(&invite_id).await?;
        }
        Commands::ShareRevoke { invite_id } => {
            commands::share_revoke_command(&invite_id).await?;
        }
        Commands::ShareUpdate {
            invite_id,
            permissions,
        } => {
            commands::share_update_command(&invite_id, permissions).await?;
        }
        // Search history commands
        Commands::HistoryList { limit } => {
            commands::history_list_command(limit).await?;
        }
        Commands::HistoryClear => {
            commands::history_clear_command().await?;
        }
        Commands::HistoryDelete { history_id } => {
            commands::history_delete_command(&history_id).await?;
        }
        // AI insight commands
        Commands::InsightReview {
            entity_type,
            entity_id,
        } => {
            commands::insight_review_command(&entity_type, &entity_id).await?;
        }
        Commands::InsightApply { insight_id } => {
            commands::insight_apply_command(&insight_id).await?;
        }
        Commands::InsightFeedback {
            insight_id,
            rating,
            comment,
        } => {
            commands::insight_feedback_command(&insight_id, rating, comment).await?;
        }
        Commands::InsightList { entity_type, limit } => {
            commands::insight_list_command(entity_type, limit).await?;
        }
    }

    Ok(())
}
