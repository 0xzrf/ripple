use anyhow::Result;

pub enum Errs {
    Get(GetErrs),
    Put,
    Del,
    Create,
    Generic,
}

pub enum GetErrs {
    Tombstone,
    Unavailable,
}
