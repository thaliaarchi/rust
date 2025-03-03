use rustc_hir::definitions::DefPathData;
use rustc_middle::mir::{Body, TerminatorKind};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::sym;

pub(super) struct TrackFormatArgs;

impl<'tcx> crate::MirPass<'tcx> for TrackFormatArgs {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let local_decls = &body.local_decls;
        for bb in body.basic_blocks.as_mut_preserves_cfg() {
            // `fmt::Arguments::{new_const, new_v1, new_v1_formatted}` have a
            // `DefPath` of the form:
            //
            // DefPath {
            //     data: [
            //         DisambiguatedDefPathData { data: TypeNs("fmt"), disambiguator: 0 },
            //         DisambiguatedDefPathData { data: Impl, disambiguator: 5 },
            //         DisambiguatedDefPathData { data: ValueNs("new_v1_formatted"), disambiguator: 0 }
            //     ],
            //     krate: crate1
            // }

            if let Some(term) = &mut bb.terminator
                && let TerminatorKind::Call { func, args, .. } = &term.kind
                && let &ty::FnDef(def_id, ..) = func.ty(local_decls, tcx).kind()
                && let path = tcx.def_path(def_id)
                && let [module_path, impl_path, method_path] = &*path.data
                && let DefPathData::TypeNs(sym::fmt) = module_path.data
                && let DefPathData::Impl = impl_path.data
                && let DefPathData::ValueNs(method_sym) = method_path.data
                && matches!(method_sym, sym::new_const | sym::new_v1 | sym::new_v1_formatted)
            {
                match (method_sym, &**args) {
                    (sym::new_const, [pieces]) => {
                        let pieces_ty = pieces.node.ty(local_decls, tcx);
                        eprintln!("[WRITE_FMT] core::fmt::Arguments::new_const({pieces_ty:?})");
                    }
                    (sym::new_v1, [pieces, args, id]) => {
                        let pieces_ty = pieces.node.ty(local_decls, tcx);
                        let args_ty = args.node.ty(local_decls, tcx);
                        let id_ty = id.node.ty(local_decls, tcx);
                        eprintln!(
                            "[WRITE_FMT] core::fmt::Arguments::new_v1({pieces_ty:?}, {args_ty:?}, {id_ty:?})"
                        );
                    }
                    (sym::new_v1_formatted, [pieces, args, fmt, id, _unsafe_arg]) => {
                        let pieces_ty = pieces.node.ty(local_decls, tcx);
                        let args_ty = args.node.ty(local_decls, tcx);
                        let fmt_ty = fmt.node.ty(local_decls, tcx);
                        let id_ty = id.node.ty(local_decls, tcx);
                        eprintln!(
                            "[WRITE_FMT] core::fmt::Arguments::new_v1_formatted({pieces_ty:?}, {args_ty:?}, {fmt_ty:?}, {id_ty:?}, UnsafeArg)"
                        );
                    }
                    _ => panic!("[WRITE_FMT] unknown arity"),
                }
            }
        }
    }

    fn is_required(&self) -> bool {
        true
    }
}
