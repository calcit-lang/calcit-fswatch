
import * as $clt from "./calcit.core.mjs";
import { calcit_dirname } from "./fswatch.$meta.mjs";
import { calcit_filename } from "./fswatch.$meta.mjs";
import { FswatchOptions } from "./fswatch.core.mjs";
import { fswatch_$x_ } from "./fswatch.core.mjs";
const _t_ = $clt.init_tags(["duration","path",]);

export function main_$x_() {
  if (arguments.length !== 0) throw $clt._args_throw('main!', 0, arguments.length);
  let tmp_AUTO_1 = $clt._$n__PCT__$M_(FswatchOptions, _t_.duration, 1000, _t_.path, "sandbox");
  let tmp_AUTO_2 = function f_PCT_(event) {
    if (arguments.length !== 1) throw $clt._args_throw('f%', 1, arguments.length);
    console.log($clt.printable(event))
  }
  ;
  return fswatch_$x_(tmp_AUTO_1, tmp_AUTO_2)
}

export function reload_$x_() {
  if (arguments.length !== 0) throw $clt._args_throw('reload!', 0, arguments.length);
}

export function run_tests() {
  if (arguments.length !== 0) throw $clt._args_throw('run-tests', 0, arguments.length);
  {
    console.log($clt.printable("%%%% test for lib"));
  }
  console.log($clt.printable(calcit_filename, calcit_dirname))
}



