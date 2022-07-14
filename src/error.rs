
pub(crate) type PanelResult<O> = Result<O, PanelError>;

pub(crate) enum PanelError {
    NoPanelCanBeDrawn,
}