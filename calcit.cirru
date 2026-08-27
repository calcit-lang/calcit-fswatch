
{} (:about "|Machine-generated snapshot. Do not edit directly — changes will be overwritten. Use `calcit query` to inspect and `calcit edit`/`calcit tree` to modify. Run `calcit docs agents --full` first. Manual edits must follow format and schema conventions, then run `calcit edit format`.") (:package |fswatch)
  :entries $ {}
    :default $ {} (:description |) (:init-fn 'fswatch.test/main!) (:mode :native) (:reload-fn 'fswatch.test/reload!)
      :feature-policy $ {}
      :modules $ []
      :type-slots $ {}
  :files $ {}
    |fswatch.core $ %{} 'FileEntry
      :defs $ {}
        |FswatchEvent $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defstruct FswatchEvent (:type 'Tag) (:path 'String) (:extra 'String)
          :examples $ []
          :schema $ :: 'StructDef
        |FswatchOptions $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defstruct FswatchOptions (:path 'String) (:duration 'Number)
          :examples $ []
          :schema $ :: 'StructDef
        |fswatch! $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defn fswatch! (options cb)
              &call-dylib-edn-fn (get-dylib-path |/dylibs/libcalcit_fswatch) |fswatch options cb
          :examples $ []
          :schema $ :: 'Fn
            {} (:return 'Unit)
              :args $ [] 'fswatch.core/FswatchOptions
                :: 'Fn $ {} (:return 'Unit)
                  :args $ [] 'fswatch.core/FswatchEvent
      :ns $ %{} 'NsEntry (:doc |)
        :code $ quote
          ns fswatch.core $ :require
            fswatch.$meta :refer $ calcit-dirname
            fswatch.util :refer $ get-dylib-path
    |fswatch.test $ %{} 'FileEntry
      :defs $ {}
        |main! $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defn main! () $ fswatch! (FswatchOptions :path |sandbox :duration 1000)
              fn (event) (println event)
          :examples $ []
          :schema $ :: 'Fn
            {} (:return 'Unit)
              :args $ []
        |reload! $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defn reload! $
          :examples $ []
          :schema $ :: 'Fn
            {} (:return 'Unit)
              :args $ []
        |run-tests $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defn run-tests () (println "|%%%% test for lib") (println calcit-filename calcit-dirname)
          :examples $ []
          :schema $ :: 'Fn
            {} (:return 'Unit)
              :args $ []
      :ns $ %{} 'NsEntry (:doc |)
        :code $ quote
          ns fswatch.test $ :require
            fswatch.core :refer $ fswatch! FswatchOptions
            fswatch.$meta :refer $ calcit-dirname calcit-filename
    |fswatch.util $ %{} 'FileEntry
      :defs $ {}
        |get-dylib-ext $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defmacro get-dylib-ext () $ case-default (&get-os) |.so (:macos |.dylib) (:windows |.dll)
          :examples $ []
          :schema $ :: 'Macro
            {}
              :capabilities $ #{} :platform-read
              :expansion $ :: 'Expr 'String
              :required $ []
        |get-dylib-path $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defn get-dylib-path (p)
              str (or-current-path calcit-dirname) p $ get-dylib-ext
          :examples $ []
          :schema $ :: 'Fn
            {} (:return 'String)
              :args $ [] 'String
        |or-current-path $ %{} 'CodeEntry (:doc |)
          :code $ quote
            defn or-current-path (p)
              if (blank? p) |. p
          :examples $ []
          :schema $ :: 'Fn
            {} (:return 'String)
              :args $ [] 'String
      :ns $ %{} 'NsEntry (:doc |)
        :code $ quote
          ns fswatch.util $ :require
            fswatch.$meta :refer $ calcit-dirname calcit-filename
