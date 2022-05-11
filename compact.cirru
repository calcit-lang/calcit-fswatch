
{} (:package |fswatch)
  :configs $ {} (:init-fn |fswatch.test/main!) (:reload-fn |fswatch.test/reload!) (:version |0.0.1)
    :modules $ []
  :entries $ {}
  :files $ {}
    |fswatch.core $ {}
      :defs $ {}
        |fswatch! $ quote
          defn fswatch! (options cb)
            &call-dylib-edn-fn (get-dylib-path "\"/dylibs/libcalcit_std") "\"fswatch" options cb
      :ns $ quote
        ns fswatch.core $ :require
          fswatch.$meta :refer $ calcit-dirname
          fswatch.util :refer $ get-dylib-path
    |fswatch.test $ {}
      :defs $ {}
        |main! $ quote
          defn main! () $ fswatch!
            {} (:path "\"sandbox") (:duration 1000)
            fn (event) (println event)
        |reload! $ quote
          defn reload! $
        |run-tests $ quote
          defn run-tests () (println "\"%%%% test for lib") (println calcit-filename calcit-dirname)
      :ns $ quote
        ns fswatch.test $ :require
          fswatch.core :refer $ fswatch!
          fswatch.$meta :refer $ calcit-dirname calcit-filename
    |fswatch.util $ {}
      :defs $ {}
        |get-dylib-ext $ quote
          defmacro get-dylib-ext () $ case-default (&get-os) "\".so" (:macos "\".dylib") (:windows "\".dll")
        |get-dylib-path $ quote
          defn get-dylib-path (p)
            str (or-current-path calcit-dirname) p $ get-dylib-ext
        |or-current-path $ quote
          defn or-current-path (p)
            if (blank? p) "\"." p
      :ns $ quote
        ns fswatch.util $ :require
          fswatch.$meta :refer $ calcit-dirname calcit-filename
