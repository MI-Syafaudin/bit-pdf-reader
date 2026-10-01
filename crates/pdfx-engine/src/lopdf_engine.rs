use crate::lopdf_document::LopdfDocumentHandle;
use crate::traits::{DocumentHandle, DocumentMutator, PdfEngine};
use lopdf::{Bookmark, Document, Object, ObjectId};
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Default, Clone)]
pub struct LopdfEngine;

impl LopdfEngine {
    pub fn new() -> Self {
        Self
    }

    /// Saves a lopdf Document to a destination path atomically using a temporary staging file.
    pub fn atomic_save(doc: &mut Document, target_path: &Path) -> Result<(), PdfxError> {
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let target_dir = target_path.parent().unwrap_or_else(|| Path::new("."));
        let mut temp = tempfile::Builder::new()
            .prefix(".pdfx-tmp-")
            .tempfile_in(target_dir)?;

        doc.save_to(&mut temp)
            .map_err(|e| PdfxError::Engine(format!("Failed to serialize PDF document: {}", e)))?;

        temp.as_file().sync_all()?;
        temp.persist(target_path)
            .map_err(|e| PdfxError::Engine(format!("Failed to persist file atomically: {}", e)))?;

        info!("Atomically saved PDF to {:?}", target_path);
        Ok(())
    }
}

impl PdfEngine for LopdfEngine {
    fn open_document(
        &self,
        path: &Path,
        password: Option<&str>,
    ) -> Result<Box<dyn DocumentHandle>, PdfxError> {
        let handle = LopdfDocumentHandle::open(path, password)?;
        Ok(Box::new(handle))
    }
}

impl DocumentMutator for LopdfEngine {
    fn merge(
        &self,
        inputs: &[PathBuf],
        output: &Path,
        cancel: &CancellationToken,
    ) -> Result<(), PdfxError> {
        if inputs.len() < 2 {
            return Err(PdfxError::InvalidOperation {
                reason: "At least 2 PDF documents are required to merge".to_string(),
            });
        }

        let mut max_id = 1;
        let mut pagenum = 1;
        let mut documents_pages = BTreeMap::new();
        let mut documents_objects = BTreeMap::new();
        let mut target_document = Document::with_version("1.7");

        for input in inputs {
            if cancel.is_cancelled() {
                return Err(PdfxError::Cancelled);
            }

            let mut doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
                path: input.clone(),
                reason: format!("Failed to read input PDF during merge: {}", e),
            })?;

            let mut first = false;
            doc.renumber_objects_with(max_id);
            max_id = doc.max_id + 1;

            for (_, object_id) in doc.get_pages() {
                if !first {
                    let bookmark =
                        Bookmark::new(format!("Page_{}", pagenum), [0.0, 0.0, 1.0], 0, object_id);
                    target_document.add_bookmark(bookmark, None);
                    first = true;
                    pagenum += 1;
                }

                if let Ok(obj) = doc.get_object(object_id) {
                    documents_pages.insert(object_id, obj.clone());
                }
            }

            documents_objects.extend(doc.objects);
        }

        let mut catalog_object: Option<(ObjectId, Object)> = None;
        let mut pages_object: Option<(ObjectId, Object)> = None;

        for (object_id, object) in documents_objects.iter() {
            match object.type_name().unwrap_or("") {
                "Catalog" => {
                    if catalog_object.is_none() {
                        catalog_object = Some((*object_id, object.clone()));
                    }
                }
                "Pages" => {
                    if let Ok(dict) = object.as_dict() {
                        let mut new_dict = dict.clone();
                        if let Some((_, ref old_obj)) = pages_object {
                            if let Ok(old_dict) = old_obj.as_dict() {
                                new_dict.extend(old_dict);
                            }
                        }
                        pages_object = Some((
                            pages_object.map(|(id, _)| id).unwrap_or(*object_id),
                            Object::Dictionary(new_dict),
                        ));
                    }
                }
                "Page" | "Outlines" | "Outline" => {}
                _ => {
                    target_document.objects.insert(*object_id, object.clone());
                }
            }
        }

        let pages_obj = pages_object
            .ok_or_else(|| PdfxError::Engine("Pages root not found in inputs".to_string()))?;
        let catalog_obj = catalog_object
            .ok_or_else(|| PdfxError::Engine("Catalog root not found in inputs".to_string()))?;

        for (object_id, object) in documents_pages.iter() {
            if let Ok(dict) = object.as_dict() {
                let mut new_dict = dict.clone();
                new_dict.set("Parent", pages_obj.0);
                target_document
                    .objects
                    .insert(*object_id, Object::Dictionary(new_dict));
            }
        }

        if let Ok(dict) = pages_obj.1.as_dict() {
            let mut new_dict = dict.clone();
            new_dict.set("Count", documents_pages.len() as u32);
            new_dict.set(
                "Kids",
                documents_pages
                    .keys()
                    .map(|id| Object::Reference(*id))
                    .collect::<Vec<_>>(),
            );
            target_document
                .objects
                .insert(pages_obj.0, Object::Dictionary(new_dict));
        }

        if let Ok(dict) = catalog_obj.1.as_dict() {
            let mut new_dict = dict.clone();
            new_dict.set("Pages", pages_obj.0);
            new_dict.remove(b"Outlines");
            target_document
                .objects
                .insert(catalog_obj.0, Object::Dictionary(new_dict));
        }

        target_document.trailer.set("Root", catalog_obj.0);
        target_document
            .trailer
            .set("Size", target_document.max_id + 1);
        target_document.renumber_objects();

        if cancel.is_cancelled() {
            return Err(PdfxError::Cancelled);
        }

        Self::atomic_save(&mut target_document, output)?;
        Ok(())
    }

    fn split(
        &self,
        input: &Path,
        ranges: &[PageRange],
        output_dir: &Path,
        cancel: &CancellationToken,
    ) -> Result<Vec<PathBuf>, PdfxError> {
        let doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
            path: input.to_path_buf(),
            reason: format!("Failed to read PDF for splitting: {}", e),
        })?;

        let total_pages = doc.get_pages().len();
        std::fs::create_dir_all(output_dir)?;

        let stem = input
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".to_string());

        let mut outputs = Vec::new();
        for (i, range) in ranges.iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(PdfxError::Cancelled);
            }

            let pages_to_keep = range.resolve(total_pages)?;
            if pages_to_keep.is_empty() {
                continue;
            }

            let mut split_doc = doc.clone();
            let all_pages: std::collections::HashSet<usize> = (1..=total_pages).collect();
            let keep_set: std::collections::HashSet<usize> =
                pages_to_keep.iter().copied().collect();
            let to_delete: Vec<u32> = all_pages.difference(&keep_set).map(|p| *p as u32).collect();

            split_doc.delete_pages(&to_delete);
            split_doc.renumber_objects();

            let out_file = output_dir.join(format!("{}_part_{}.pdf", stem, i + 1));
            Self::atomic_save(&mut split_doc, &out_file)?;
            outputs.push(out_file);
        }

        Ok(outputs)
    }

    fn rotate(
        &self,
        input: &Path,
        pages: &PageRange,
        degrees: u16,
        output: &Path,
    ) -> Result<(), PdfxError> {
        if degrees != 90 && degrees != 180 && degrees != 270 {
            return Err(PdfxError::InvalidRotation { degrees });
        }

        let mut doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
            path: input.to_path_buf(),
            reason: format!("Failed to load PDF for rotation: {}", e),
        })?;

        let total_pages = doc.get_pages().len();
        let target_pages = pages.resolve(total_pages)?;

        let page_map = doc.get_pages();
        for page_num in target_pages {
            if let Some(&page_id) = page_map.get(&(page_num as u32)) {
                if let Ok(page_obj) = doc.get_object_mut(page_id) {
                    if let Ok(dict) = page_obj.as_dict_mut() {
                        let current_rot = dict
                            .get(b"Rotate")
                            .ok()
                            .and_then(|o| o.as_i64().ok())
                            .unwrap_or(0);
                        let new_rot = (current_rot + degrees as i64).rem_euclid(360);
                        dict.set("Rotate", Object::Integer(new_rot));
                        debug!("Rotated page {} to {} degrees", page_num, new_rot);
                    }
                }
            }
        }

        Self::atomic_save(&mut doc, output)?;
        Ok(())
    }

    fn delete_pages(
        &self,
        input: &Path,
        pages: &PageRange,
        output: &Path,
    ) -> Result<(), PdfxError> {
        let mut doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
            path: input.to_path_buf(),
            reason: format!("Failed to load PDF for deleting pages: {}", e),
        })?;

        let total_pages = doc.get_pages().len();
        let to_remove = pages.resolve(total_pages)?;

        if to_remove.len() >= total_pages {
            return Err(PdfxError::InvalidOperation {
                reason: "Cannot delete all pages in the document".to_string(),
            });
        }

        let to_delete_u32: Vec<u32> = to_remove.into_iter().map(|p| p as u32).collect();
        doc.delete_pages(&to_delete_u32);
        doc.renumber_objects();

        Self::atomic_save(&mut doc, output)?;
        Ok(())
    }

    fn reorder_pages(
        &self,
        input: &Path,
        new_order: &[usize],
        output: &Path,
    ) -> Result<(), PdfxError> {
        let mut doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
            path: input.to_path_buf(),
            reason: format!("Failed to load PDF for reordering: {}", e),
        })?;

        let total_pages = doc.get_pages().len();
        if new_order.len() != total_pages {
            return Err(PdfxError::InvalidOperation {
                reason: format!(
                    "New page order count ({}) does not match document page count ({})",
                    new_order.len(),
                    total_pages
                ),
            });
        }

        for &p in new_order {
            if p == 0 || p > total_pages {
                return Err(PdfxError::PageOutOfBounds {
                    index: p,
                    total: total_pages,
                });
            }
        }

        let old_pages = doc.get_pages();
        let reordered_ids: Vec<ObjectId> = new_order
            .iter()
            .filter_map(|&p| old_pages.get(&(p as u32)).copied())
            .collect();

        // Update Pages dictionary
        for object in doc.objects.values_mut() {
            if let Ok(dict) = object.as_dict_mut() {
                if dict.get(b"Type").ok().and_then(|o| o.as_name_str().ok()) == Some("Pages") {
                    dict.set(
                        "Kids",
                        reordered_ids
                            .iter()
                            .map(|id| Object::Reference(*id))
                            .collect::<Vec<_>>(),
                    );
                    dict.set("Count", reordered_ids.len() as u32);
                }
            }
        }

        doc.renumber_objects();
        Self::atomic_save(&mut doc, output)?;
        Ok(())
    }

    fn extract_pages(
        &self,
        input: &Path,
        pages: &PageRange,
        output: &Path,
    ) -> Result<(), PdfxError> {
        let mut doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
            path: input.to_path_buf(),
            reason: format!("Failed to load PDF for extracting pages: {}", e),
        })?;

        let total_pages = doc.get_pages().len();
        let pages_to_keep = pages.resolve(total_pages)?;
        if pages_to_keep.is_empty() {
            return Err(PdfxError::InvalidOperation {
                reason: "No matching pages found to extract".to_string(),
            });
        }

        let keep_set: std::collections::HashSet<usize> = pages_to_keep.into_iter().collect();
        let to_delete: Vec<u32> = (1..=total_pages)
            .filter(|p| !keep_set.contains(p))
            .map(|p| p as u32)
            .collect();

        doc.delete_pages(&to_delete);
        doc.renumber_objects();

        Self::atomic_save(&mut doc, output)?;
        Ok(())
    }

    fn compress(&self, input: &Path, output: &Path) -> Result<(), PdfxError> {
        let mut doc = Document::load(input).map_err(|e| PdfxError::CorruptedDocument {
            path: input.to_path_buf(),
            reason: format!("Failed to load PDF for compression: {}", e),
        })?;

        doc.prune_objects();
        doc.delete_zero_length_streams();
        doc.compress();

        Self::atomic_save(&mut doc, output)?;
        Ok(())
    }
}
