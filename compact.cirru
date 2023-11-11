
{} (:package |fswatch)
  :configs $ {} (:init-fn |fswatch.test/main!) (:reload-fn |fswatch.test/reload!) (:version |0.0.3)
    :modules $ []
  :entries $ {}
  :files $ {}
    |fswatch.core $ %{} :FileEntry
      :defs $ {}
        |fswatch! $ %{} :CodeEntry (:doc |)
          :code $ quote
            defn fswatch! (options cb)
              &call-dylib-edn-fn (get-dylib-path "\"/dylibs/libcalcit_fswatch") "\"fswatch" options cb
      :ns $ %{} :CodeEntry (:doc |)
        :code $ quote
          ns fswatch.core $ :require
            fswatch.$meta :refer $ calcit-dirname
            fswatch.util :refer $ get-dylib-path
    |fswatch.test $ %{} :FileEntry
      :defs $ {}
        |main! $ %{} :CodeEntry (:doc |)
          :code $ quote
            defn main! () $ fswatch!
              {} (:path "\"sandbox") (:duration 1000)
              fn (event) (println event)
        |reload! $ %{} :CodeEntry (:doc |)
          :code $ quote
            defn reload! $
        |run-tests $ %{} :CodeEntry (:doc |)
          :code $ quote
            defn run-tests () (println "\"%%%% test for lib") (println calcit-filename calcit-dirname)
      :ns $ %{} :CodeEntry (:doc |)
        :code $ quote
          ns fswatch.test $ :require
            fswatch.core :refer $ fswatch!
            fswatch.$meta :refer $ calcit-dirname calcit-filename
    |fswatch.util $ %{} :FileEntry
      :defs $ {}
        |get-dylib-ext $ %{} :CodeEntry (:doc |)
          :code $ quote
            defmacro get-dylib-ext () $ case-default (&get-os) "\".so" (:macos "\".dylib") (:windows "\".dll")
        |get-dylib-path $ %{} :CodeEntry (:doc |)
          :code $ quote
            defn get-dylib-path (p)
              str (or-current-path calcit-dirname) p $ get-dylib-ext
        |or-current-path $ %{} :CodeEntry (:doc |)
          :code $ quote
            defn or-current-path (p)
              if (blank? p) "\"." p
      :ns $ %{} :CodeEntry (:doc |)
        :code $ quote
          ns fswatch.util $ :require
            fswatch.$meta :refer $ calcit-dirname calcit-filename
