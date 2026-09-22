
import * as $clt from "./calcit.core.mjs";
import { get_dylib_path } from "./fswatch.util.mjs";
const _t_ = $clt.init_tags(["FswatchEvent","FswatchOptions","duration","extra","path","type",]);

export function fswatch_$x_(options, cb) {
  if (arguments.length !== 2) throw $clt._args_throw('fswatch!', 2, arguments.length);
  let tmp_AUTO_1 = get_dylib_path("/dylibs/libcalcit_fswatch");
  return $clt._$n_call_dylib_edn_fn(tmp_AUTO_1, "fswatch", options, cb)
}



export var FswatchEvent = $clt._$n_struct_def_$o_new(_t_.FswatchEvent, $clt._$L_(_t_.type, new $clt.CalcitSymbol("Tag")), $clt._$L_(_t_.path, new $clt.CalcitSymbol("String")), $clt._$L_(_t_.extra, new $clt.CalcitSymbol("String")));

export var FswatchOptions = $clt._$n_struct_def_$o_new(_t_.FswatchOptions, $clt._$L_(_t_.path, new $clt.CalcitSymbol("String")), $clt._$L_(_t_.duration, new $clt.CalcitSymbol("Number")));

