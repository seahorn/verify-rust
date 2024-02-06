configure_file(sea.yaml sea.yaml @ONLY)
configure_file(sea_cex_base.yaml sea.cex.yaml @ONLY)
configure_file(sea_vac.yaml sea.vac.yaml @ONLY)
configure_file(sea_pcond.yaml sea.pcond.yaml @ONLY)


add_subdirectory(sea-lib)


# Add "Jobs"
add_subdirectory(rust-jobs)
