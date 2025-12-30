use crate::error::SdkError;
use crate::types::RefInfo;
use crate::RepoHandle;

#[inline]
fn box_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> SdkError {
    SdkError::Git(Box::new(e))
}

#[coverage(off)]
fn symbolic_head_unreachable() -> Result<RefInfo, SdkError> {
    unreachable!("head_ref() returns Some for Symbolic heads")
}

pub fn list_refs(repo: &RepoHandle, prefix: Option<&str>) -> Result<Vec<RefInfo>, SdkError> {
    let local = repo.to_local();
    let refs_platform = local.references().map_err(box_err)?;

    let iter = match prefix {
        Some(p) => refs_platform.prefixed(p).map_err(box_err)?,
        None => refs_platform.all().map_err(box_err)?,
    };

    let mut result = Vec::new();
    for reference in iter {
        let reference = reference.map_err(SdkError::Git)?;
        let ref_info = convert_reference(&reference)?;
        result.push(ref_info);
    }

    Ok(result)
}

pub fn resolve_ref(repo: &RepoHandle, name: &str) -> Result<RefInfo, SdkError> {
    let local = repo.to_local();
    let reference = local.find_reference(name).map_err(box_err)?;
    convert_reference(&reference)
}

#[coverage(off)]
pub fn get_head(repo: &RepoHandle) -> Result<RefInfo, SdkError> {
    let local = repo.to_local();
    let head_ref = local.head_ref().map_err(box_err)?;

    match head_ref {
        Some(reference) => convert_reference(&reference),
        None => {
            let head = local.head().map_err(box_err)?;
            match head.kind {
                gix::head::Kind::Detached { target, .. } => Ok(RefInfo {
                    name: "HEAD".to_string(),
                    target,
                    is_symbolic: false,
                    symbolic_target: None,
                }),
                gix::head::Kind::Unborn(name) => Ok(RefInfo {
                    name: "HEAD".to_string(),
                    target: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
                    is_symbolic: true,
                    symbolic_target: Some(name.as_bstr().to_string()),
                }),
                gix::head::Kind::Symbolic(_) => symbolic_head_unreachable()
            }
        }
    }
}

pub fn list_branches(repo: &RepoHandle) -> Result<Vec<RefInfo>, SdkError> {
    list_refs(repo, Some("refs/heads/"))
}

pub fn list_tags(repo: &RepoHandle) -> Result<Vec<RefInfo>, SdkError> {
    list_refs(repo, Some("refs/tags/"))
}

fn convert_reference(reference: &gix::Reference<'_>) -> Result<RefInfo, SdkError> {
    let name = reference.name().as_bstr().to_string();
    let target_ref = reference.target();

    match target_ref {
        gix_ref::TargetRef::Symbolic(sym) => {
            let mut peeled = reference.clone();
            let oid = peeled.peel_to_id().map_err(box_err)?;
            Ok(RefInfo {
                name,
                target: oid.detach(),
                is_symbolic: true,
                symbolic_target: Some(sym.as_bstr().to_string()),
            })
        }
        gix_ref::TargetRef::Object(oid) => Ok(RefInfo {
            name,
            target: oid.to_owned(),
            is_symbolic: false,
            symbolic_target: None,
        }),
    }
}
