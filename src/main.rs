// main.rs
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::web::{FormConfig, JsonConfig};
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;

// use crate::controllers::image_optimizer::optimize_image;

mod auth;
mod controllers;
mod db;
mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("starting up...");
    let pool = match db::establish_connection().await {
        Ok(pool) => pool,
        Err(e) => {
            log::error!("Gagal inisialisasi pool database: {:?}", e);
            std::process::exit(1);
        }
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
            .supports_credentials()
            .max_age(3600);

        let json_config = JsonConfig::default()
            .limit(50 * 1024 * 1024) // 50MB untuk JSON
            .content_type_required(false) // Kadang header content-type tidak tepat
            .error_handler(|err, _req| {
                log::error!("JSON payload error: {}", err);
                actix_web::error::ErrorBadRequest(format!("Payload error: {}", err))
            });

        // Untuk Form data
        let form_config = FormConfig::default()
            .limit(50 * 1024 * 1024) // 50MB untuk form
            .error_handler(|err, _req| {
                log::error!("Form payload error: {}", err);
                actix_web::error::ErrorBadRequest(format!("Form error: {}", err))
            });

        // Untuk raw payload
        let payload_config = web::PayloadConfig::new(50 * 1024 * 1024).limit(50 * 1024 * 1024);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(json_config)
            .app_data(form_config)
            .app_data(payload_config)
            .wrap(cors)
            .wrap(Logger::default())
            .service(Files::new("/uploads", "./uploads").show_files_listing())
            .service(controllers::preview::preview)
            //home_controller
            .service(controllers::home_controller::get_data_setting)
            .service(controllers::home_controller::get_gallery)
            .service(controllers::home_controller::get_all_gallery)
            .service(controllers::home_controller::get_gallery_by_id)
            .service(controllers::home_controller::get_video)
            .service(controllers::home_controller::get_berita)
            .service(controllers::home_controller::get_all_berita)
            .service(controllers::home_controller::get_random_berita)
            .service(controllers::home_controller::get_berita_by_slug)
            .service(controllers::home_controller::get_kegiatan)
            .service(controllers::home_controller::get_kegiatan_slug)
            .service(controllers::home_controller::get_profil)
            .service(controllers::home_controller::get_pdp)
            .service(controllers::home_controller::get_pdp_kabupaten)
            .service(controllers::home_controller::get_pdp_provinsi)
            .service(controllers::home_controller::get_kabupaten)
            .service(controllers::home_controller::get_provinsi)
            .service(controllers::home_controller::get_pelaksana_pusat)
            .service(controllers::home_controller::pelaksana_provinsi)
            .service(controllers::home_controller::get_pelaksana_provinsi)
            .service(controllers::home_controller::pelaksana_kabupaten_all_provinsi)
            .service(controllers::home_controller::get_pelaksana_kabupaten_names)
            .service(controllers::home_controller::get_pelaksana_kabupaten)
            .service(controllers::home_controller::get_regulasi)
            .service(controllers::home_controller::view_regulasi)
            .service(controllers::home_controller::post_pesan)
            .service(controllers::home_controller::get_pengumuman)
            //auth Controller
            .service(controllers::auth_controller::get_jabatan)
            .service(controllers::auth_controller::get_hobi)
            .service(controllers::auth_controller::get_minat)
            .service(controllers::auth_controller::get_detail_minat)
            .service(controllers::auth_controller::get_bakat)
            .service(controllers::auth_controller::get_detail_bakat)
            .service(controllers::auth_controller::register_user)
            .service(controllers::auth_controller::get_user)
            .service(controllers::auth_controller::login)
            .service(controllers::auth_controller::forgot_password)
            .service(controllers::auth_controller::reset_password)
            .service(controllers::auth_controller::logout)

            // .service(optimize_image)
            .service(controllers::dashboard_controller::get_contact)
            .service(controllers::dashboard_controller::delete_contact)
            .service(controllers::dashboard_controller::get_pdp_terdaftar)
            .service(controllers::dashboard_controller::get_pdp_belum_diverifikasi)
            .service(controllers::dashboard_controller::get_pdp_diverifikasi)
            .service(controllers::dashboard_controller::get_pdp_simental)
            .service(controllers::dashboard_controller::reply_contact)
            //post_controller
            .service(controllers::post_controller::get_category)
            .service(controllers::post_controller::get_post_by_id)
            .service(controllers::post_controller::create_post)
            .service(controllers::post_controller::update_post)
            .service(controllers::post_controller::delete_post)
            .service(controllers::post_controller::admin_get_all_berita)
            //pengumuman_controller
            .service(controllers::pengumuman_controller::put_announcement)
            .service(controllers::pengumuman_controller::delete_pengumuman)
            .service(controllers::pengumuman_controller::create_pengumuman)
            //video_controller
            .service(controllers::video_controller::get_video)
            .service(controllers::video_controller::update_video)
            //kesbangpol
            .service(controllers::kesbangpol_controller::kesbangpol_get_pdp_terdaftar)
            .service(controllers::kesbangpol_controller::kesbangpol_get_pdp_belum_diverifikasi)
            .service(controllers::kesbangpol_controller::kesbangpol_get_pdp_diverifikasi)
            .service(controllers::kesbangpol_controller::kesbangpol_list_pdp_belum_registrasi)
            .service(controllers::kesbangpol_controller::kesbangpol_list_pdp_belum_diverifikasi)
            .service(controllers::kesbangpol_controller::kesbangpol_list_pdp_verified)
            .service(controllers::kesbangpol_controller::kesbangpol_list_pdp_simental)
            .service(controllers::kesbangpol_controller::kesbangpol_list_pdp_tidak_aktif)
            .service(controllers::kesbangpol_controller::kesbangpol_get_pelaksana_provinsi)
            .service(controllers::kesbangpol_controller::kesbangpol_get_pelaksana_kabupaten)
            .service(controllers::kesbangpol_controller::get_provinsi_by_id)
            .service(controllers::kesbangpol_controller::get_kabupaten_by_id)
            .service(controllers::kesbangpol_controller::get_kabupaten_by_provinsi)
            .service(controllers::kesbangpol_controller::pdp_kesbangpol_belum_registrasi_all)
            .service(controllers::kesbangpol_controller::pdp_kesbangpol_belum_diverifikasi_all)
            .service(controllers::kesbangpol_controller::pdp_kesbangpol_verified_all)
            .service(controllers::kesbangpol_controller::pdp_kesbangpol_simental_all)
            .service(controllers::kesbangpol_controller::pdp_kesbangpol_tidak_aktif_all)
            //pelaksana
            .service(controllers::pelaksana_controller::get_jabatan)
            .service(controllers::pelaksana_controller::get_jabatan_provinsi)
            .service(controllers::pelaksana_controller::get_jabatan_kabupaten)
            .service(controllers::pelaksana_controller::get_pelaksana_pusat)
            .service(controllers::pelaksana_controller::get_pelaksana_provinsi)
            .service(controllers::pelaksana_controller::get_pelaksana_kabupaten)
            .service(controllers::pelaksana_controller::update_pelaksana_pusat)
            .service(controllers::pelaksana_controller::update_pelaksana_provinsi)
            .service(controllers::pelaksana_controller::update_pelaksana_kabupaten)
            .service(controllers::pelaksana_controller::create_pelaksana_pusat)
            .service(controllers::pelaksana_controller::create_pelaksana_provinsi)
            .service(controllers::pelaksana_controller::create_pelaksana_kabupaten)
            .service(controllers::pelaksana_controller::delete_pelaksana_pusat)
            .service(controllers::pelaksana_controller::delete_pelaksana_provinsi)
            .service(controllers::pelaksana_controller::delete_pelaksana_kabupaten)
            .service(controllers::pelaksana_controller::list_pdp_by_claims)
            .service(controllers::pelaksana_controller::get_all_pelaksana_provinsi)
            .service(controllers::pelaksana_controller::get_all_pelaksana_kabupaten)
            //pendidikan
            .service(controllers::userpanel::get_pendidikan)
            .service(controllers::userpanel::add_pendidikan)
            .service(controllers::userpanel::update_pendidikan)
            .service(controllers::userpanel::delete_pendidikan)
            //diklat
            .service(controllers::userpanel::get_diklat)
            .service(controllers::userpanel::add_diklat)
            .service(controllers::userpanel::update_diklat)
            .service(controllers::userpanel::delete_diklat)
            //penghargaan
            .service(controllers::userpanel::get_penghargaan)
            .service(controllers::userpanel::add_penghargaan)
            .service(controllers::userpanel::update_penghargaan)
            .service(controllers::userpanel::delete_penghargaan)
            //organisasi
            .service(controllers::userpanel::get_organisasi)
            .service(controllers::userpanel::add_organisasi)
            .service(controllers::userpanel::update_organisasi)
            .service(controllers::userpanel::delete_organisasi)
            //kegiatan
            .service(controllers::userpanel::get_kegiatan)
            //ketum
            .service(controllers::userpanel::get_ketum)
            //idCard & CV
            .service(controllers::cv_card_controller::get_ketum_id_card)
            .service(controllers::cv_card_controller::get_organisasi_cv)
            .service(controllers::cv_card_controller::get_pendidikan_cv)
            .service(controllers::cv_card_controller::get_pdp_cv)
            //user
            .service(controllers::user_controller::update_profile_user)
            .service(controllers::user_controller::change_password)
            .service(controllers::user_controller::get_current_user)
            .service(controllers::user_controller::get_current_pelaksana)
            .service(controllers::user_controller::get_current_pelaksana_dynamic)
            .service(controllers::user_controller::create_pelaksana)
            .service(controllers::user_controller::update_pelaksana)
            .service(controllers::user_controller::get_all_user)
            .service(controllers::user_controller::update_user_by_id)
            .service(controllers::user_controller::delete_user)
            .service(controllers::user_controller::new_add_user)
            //gallery
            .service(controllers::gallery_controller::create_gallery)
            .service(controllers::gallery_controller::update_gallery_meta_spoof)
            .service(controllers::gallery_controller::append_gallery_photos)
            .service(controllers::gallery_controller::delete_one_photo)
            .service(controllers::gallery_controller::delete_gallery)
            //Kegiatan
            .service(controllers::kegiatan_controller::update_kegiatan)
            .service(controllers::kegiatan_controller::create_kegiatan)
            .service(controllers::kegiatan_controller::delete_kegiatan)
            //rating
            .service(controllers::rating_controller::submit_rating)
            .service(controllers::rating_controller::get_rating_stats)
            .service(controllers::rating_controller::get_ratings)
            .service(controllers::rating_controller::approve_rating)
            .service(controllers::rating_controller::delete_rating)
            //regulasi
            .service(controllers::regulasi_controller::update_regulasi)
            .service(controllers::regulasi_controller::create_regulasi)
            .service(controllers::regulasi_controller::delete_regulasi)
            //visitor
            .service(controllers::visitor_advanced_controller::get_advanced_stats)
            .service(controllers::visitor_controller::track_visitor)
            .service(controllers::visitor_controller::get_stats)
            .service(controllers::visitor_controller::get_recent_visitors)
            .service(controllers::visitor_controller::get_visitor_by_session)
            .service(controllers::visitor_controller::get_stats_summary)
            .service(controllers::visitor_controller::get_stats_summary2)
            //pendaftaran
            .service(controllers::pendaftaran_dppi_controller::download_to_excel)
            .service(controllers::pendaftaran_dppi_controller::get_pendaftaran_list)
            .service(controllers::pendaftaran_dppi_controller::get_stats)
            .service(controllers::pendaftaran_dppi_controller::create_pendaftaran)
            .service(controllers::pendaftaran_dppi_controller::get_pendaftaran_by_id)
            .service(controllers::pendaftaran_dppi_controller::upload_document)
            .service(controllers::pendaftaran_dppi_controller::update_status)
            .service(controllers::pendaftaran_dppi_controller::delete_pendaftaran)
            .service(controllers::pendaftaran_dppi_controller::download_document)
            .service(controllers::pendaftaran_dppi_controller::upload_rekomendasi)
            .service(controllers::pendaftaran_dppi_controller::download_rekomendasi)
            //pendaftaran Provinsi
            .service(controllers::pendaftaran_dppi_controller_provinsi::download_to_excel_provinsi)
            .service(
                controllers::pendaftaran_dppi_controller_provinsi::get_pendaftaran_list_provinsi,
            )
            .service(controllers::pendaftaran_dppi_controller_provinsi::get_stats_provinsi)
            .service(controllers::pendaftaran_dppi_controller_provinsi::create_pendaftaran_provinsi)
            .service(
                controllers::pendaftaran_dppi_controller_provinsi::get_pendaftaran_by_id_provinsi,
            )
            .service(controllers::pendaftaran_dppi_controller_provinsi::upload_document_provinsi)
            .service(controllers::pendaftaran_dppi_controller_provinsi::update_status_provinsi)
            .service(controllers::pendaftaran_dppi_controller_provinsi::delete_pendaftaran_provinsi)
            .service(controllers::pendaftaran_dppi_controller_provinsi::download_document_provinsi)
            .service(controllers::pendaftaran_dppi_controller_provinsi::upload_rekomendasi_provinsi)
            .service(
                controllers::pendaftaran_dppi_controller_provinsi::download_rekomendasi_provinsi,
            )
            //surat rekomendasi
            .service(controllers::surat_rekomendasi_controller::upload_surat_rekomendasi)
            .service(controllers::surat_rekomendasi_controller::get_surat_rekomendasi_list)
            .service(controllers::surat_rekomendasi_controller::download_surat_rekomendasi)
            .service(controllers::surat_rekomendasi_controller::delete_surat_rekomendasi)
            //majelis pertimbangan
            .service(controllers::majelis_pertimbangan_controller::get_all_majelis_pertimbangan)
            .service(controllers::majelis_pertimbangan_controller::create_majelis_pertimbangan)
            .service(controllers::majelis_pertimbangan_controller::update_majelis_pertimbangan)
            .service(controllers::majelis_pertimbangan_controller::delete_majelis_pertimbangan)
            .service(controllers::majelis_pertimbangan_controller::get_all_mp)
            //pdp
            .service(controllers::pdp_controller::scope())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
