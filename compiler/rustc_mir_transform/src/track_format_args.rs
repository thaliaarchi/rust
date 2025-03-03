use rustc_hir::LangItem;
use rustc_middle::mir::{Body, TerminatorKind};
use rustc_middle::ty::{self, TyCtxt};

pub(super) struct TrackFormatArgs;

impl<'tcx> crate::MirPass<'tcx> for TrackFormatArgs {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        eprintln!("[WRITE_FMT] Starting TrackFormatArgs");
        for bb in body.basic_blocks.as_mut_preserves_cfg() {
            if let Some(term) = &mut bb.terminator
                && let TerminatorKind::Call { func, args: _, .. } = &term.kind
                && let &ty::FnDef(def_id, ..) = func.ty(&body.local_decls, tcx).kind()
                && tcx.is_lang_item(def_id, LangItem::FormatArguments)
            {
                // let path = tcx.def_path(def_id);
                eprintln!("[WRITE_FMT] Found LangItem::FormatArguments");
            }
        }
    }

    fn is_required(&self) -> bool {
        true
    }
}
