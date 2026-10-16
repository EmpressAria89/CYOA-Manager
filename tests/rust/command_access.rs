mod command_access_tests {
 use super::*;
 #[test] fn only_main_window_may_run_manager_commands(){
  let healthy=StartupFailure(None);assert!(command_error("main",&healthy).is_none());
  for label in ["viewer-1234","website-1234","recovery-1234","main-other",""]{assert!(command_error(label,&healthy).is_some(),"{label}");}
 }
 #[test]fn failed_startup_blocks_every_window(){
  let failed=StartupFailure(Some("Library could not be read".into()));
  for label in ["main","viewer-1234","website-1234"]{assert_eq!(command_error(label,&failed).as_deref(),Some("Library could not be read"));}
 }
}
