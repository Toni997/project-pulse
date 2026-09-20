use bitflags::bitflags;

bitflags! {
    /// Which part(s) of the derived snapshot a project mutation invalidates -
    /// e.g. moving a clip only invalidates the scheduler, so there's no reason
    /// to also rebuild the render graph. Combine with `|`; `RebuildScope::all()`
    /// and `::empty()` come for free.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RebuildScope: u8 {
        const SCHEDULER = 1 << 0;
        const RENDER_GRAPH = 1 << 1;
        const DATA_NODES = 1 << 2;
    }
}
