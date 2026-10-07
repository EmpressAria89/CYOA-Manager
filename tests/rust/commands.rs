#[cfg(test)]
mod thumbnail_tests {
 use super::*;
 #[test]fn cover_fallback_checks_later_rows_and_choices(){
  let mut rows=vec![serde_json::json!({"image":""});6];rows.push(serde_json::json!({"image":"later.png","objects":[{"image":"choice.png"}]}));
  assert_eq!(extract_cover_image(&serde_json::json!({"rows":rows})).as_deref(),Some("later.png"));
  assert_eq!(extract_cover_image(&serde_json::json!({"rows":[{"objects":[{"image":"choice.png"}]}]})).as_deref(),Some("choice.png"));
 }
 #[test]fn thumbnail_has_strict_budget_and_preserves_source_bytes(){
  let image=image::RgbImage::from_fn(1024,1024,|x,y|{let value=x.wrapping_mul(747796405).wrapping_add(y.wrapping_mul(2891336453));image::Rgb([value as u8,(value>>8)as u8,(value>>16)as u8])});
  let mut source=std::io::Cursor::new(Vec::new());DynamicImage::ImageRgb8(image).write_to(&mut source,image::ImageFormat::Png).unwrap();let original=source.into_inner();let before=original.clone();
  let thumb=compress_thumbnail(&original,60*1024).unwrap();assert!(thumb.len()<=60*1024);let decoded=image::load_from_memory(&thumb).unwrap();assert!(decoded.width()<=512&&decoded.height()<=512);assert_eq!(original,before);
 }
}
