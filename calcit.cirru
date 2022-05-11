
{}
  :configs $ {} (:init-fn |fswatch.test/main!) (:port 6001) (:reload-fn |fswatch.test/reload!) (:version |0.0.1)
    :modules $ []
  :entries $ {}
  :ir $ {} (:package |fswatch)
    :files $ {}
      |fswatch.core $ {}
        :configs $ {}
        :defs $ {}
          |fswatch! $ {} (:at 1630219258753) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1630219258753) (:by |u0) (:text |defn) (:type :leaf)
              |j $ {} (:at 1652238095321) (:by |u0) (:text |fswatch!) (:type :leaf)
              |r $ {} (:at 1630219268038) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1652238099382) (:by |u0) (:text |options) (:type :leaf)
                  |b $ {} (:at 1652238713059) (:by |u0) (:text |cb) (:type :leaf)
              |v $ {} (:at 1630219268038) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1652238705446) (:by |u0) (:text |&call-dylib-edn-fn) (:type :leaf)
                  |b $ {} (:at 1634804189975) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1634804196083) (:by |u0) (:text |get-dylib-path) (:type :leaf)
                      |j $ {} (:at 1634804199741) (:by |u0) (:text "|\"/dylibs/libcalcit_std") (:type :leaf)
                  |r $ {} (:at 1652238119083) (:by |u0) (:text "|\"fswatch") (:type :leaf)
                  |t $ {} (:at 1652238314480) (:by |u0) (:text |options) (:type :leaf)
                  |u $ {} (:at 1652238715594) (:by |u0) (:text |cb) (:type :leaf)
        :ns $ {} (:at 1630171366222) (:by |u0) (:type :expr)
          :data $ {}
            |T $ {} (:at 1630171366222) (:by |u0) (:text |ns) (:type :leaf)
            |j $ {} (:at 1630171366222) (:by |u0) (:text |fswatch.core) (:type :leaf)
            |r $ {} (:at 1630175118985) (:by |u0) (:type :expr)
              :data $ {}
                |T $ {} (:at 1630175119637) (:by |u0) (:text |:require) (:type :leaf)
                |j $ {} (:at 1630175120856) (:by |u0) (:type :expr)
                  :data $ {}
                    |T $ {} (:at 1634703660055) (:by |u0) (:text |fswatch.$meta) (:type :leaf)
                    |j $ {} (:at 1630175127717) (:by |u0) (:text |:refer) (:type :leaf)
                    |r $ {} (:at 1630175128076) (:by |u0) (:type :expr)
                      :data $ {}
                        |T $ {} (:at 1630175130627) (:by |u0) (:text |calcit-dirname) (:type :leaf)
                |r $ {} (:at 1633181140100) (:by |u0) (:type :expr)
                  :data $ {}
                    |T $ {} (:at 1634703662332) (:by |u0) (:text |fswatch.util) (:type :leaf)
                    |j $ {} (:at 1633181140100) (:by |u0) (:text |:refer) (:type :leaf)
                    |r $ {} (:at 1633181140100) (:by |u0) (:type :expr)
                      :data $ {}
                        |T $ {} (:at 1634804181370) (:by |u0) (:text |get-dylib-path) (:type :leaf)
        :proc $ {} (:at 1630171366222) (:by |u0) (:type :expr)
          :data $ {}
      |fswatch.test $ {}
        :configs $ {}
        :defs $ {}
          |main! $ {} (:at 1633149996242) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1633149996242) (:by |u0) (:text |defn) (:type :leaf)
              |j $ {} (:at 1633149996242) (:by |u0) (:text |main!) (:type :leaf)
              |r $ {} (:at 1633149996242) (:by |u0) (:type :expr)
                :data $ {}
              |t $ {} (:at 1652238198546) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1652238203103) (:by |u0) (:text |fswatch!) (:type :leaf)
                  |b $ {} (:at 1652238207545) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1652238207838) (:by |u0) (:text |{}) (:type :leaf)
                      |b $ {} (:at 1652238209937) (:by |u0) (:type :expr)
                        :data $ {}
                          |T $ {} (:at 1652238210631) (:by |u0) (:text |:path) (:type :leaf)
                          |b $ {} (:at 1652238214038) (:by |u0) (:text "|\"sandbox") (:type :leaf)
                      |h $ {} (:at 1652238214860) (:by |u0) (:type :expr)
                        :data $ {}
                          |T $ {} (:at 1652238217459) (:by |u0) (:text |:duration) (:type :leaf)
                          |b $ {} (:at 1652238220005) (:by |u0) (:text |1000) (:type :leaf)
                  |h $ {} (:at 1652238221324) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1652238221645) (:by |u0) (:text |fn) (:type :leaf)
                      |b $ {} (:at 1652238222228) (:by |u0) (:type :expr)
                        :data $ {}
                          |T $ {} (:at 1652238223827) (:by |u0) (:text |event) (:type :leaf)
                      |h $ {} (:at 1652238224258) (:by |u0) (:type :expr)
                        :data $ {}
                          |T $ {} (:at 1652238225069) (:by |u0) (:text |println) (:type :leaf)
                          |b $ {} (:at 1652238226303) (:by |u0) (:text |event) (:type :leaf)
          |reload! $ {} (:at 1633149998862) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1633149998862) (:by |u0) (:text |defn) (:type :leaf)
              |j $ {} (:at 1633149998862) (:by |u0) (:text |reload!) (:type :leaf)
              |r $ {} (:at 1633149998862) (:by |u0) (:type :expr)
                :data $ {}
          |run-tests $ {} (:at 1633150008092) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1633150011172) (:by |u0) (:text |defn) (:type :leaf)
              |j $ {} (:at 1633150008092) (:by |u0) (:text |run-tests) (:type :leaf)
              |r $ {} (:at 1633150008092) (:by |u0) (:type :expr)
                :data $ {}
              |v $ {} (:at 1634703837934) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1634703837934) (:by |u0) (:text |println) (:type :leaf)
                  |j $ {} (:at 1634703847178) (:by |u0) (:text "|\"%%%% test for lib") (:type :leaf)
              |x $ {} (:at 1634703837934) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1634703837934) (:by |u0) (:text |println) (:type :leaf)
                  |j $ {} (:at 1634703837934) (:by |u0) (:text |calcit-filename) (:type :leaf)
                  |r $ {} (:at 1634703837934) (:by |u0) (:text |calcit-dirname) (:type :leaf)
        :ns $ {} (:at 1633149625774) (:by |u0) (:type :expr)
          :data $ {}
            |T $ {} (:at 1633149625774) (:by |u0) (:text |ns) (:type :leaf)
            |j $ {} (:at 1633149625774) (:by |u0) (:text |fswatch.test) (:type :leaf)
            |r $ {} (:at 1633149974572) (:by |u0) (:type :expr)
              :data $ {}
                |T $ {} (:at 1633149975596) (:by |u0) (:text |:require) (:type :leaf)
                |j $ {} (:at 1634703855566) (:by |u0) (:type :expr)
                  :data $ {}
                    |T $ {} (:at 1634703858564) (:by |u0) (:text |fswatch.core) (:type :leaf)
                    |j $ {} (:at 1634703859915) (:by |u0) (:text |:refer) (:type :leaf)
                    |r $ {} (:at 1634703860100) (:by |u0) (:type :expr)
                      :data $ {}
                        |T $ {} (:at 1652238175737) (:by |u0) (:text |fswatch!) (:type :leaf)
                |r $ {} (:at 1634703941759) (:by |u0) (:type :expr)
                  :data $ {}
                    |T $ {} (:at 1634703941759) (:by |u0) (:text |fswatch.$meta) (:type :leaf)
                    |j $ {} (:at 1634703941759) (:by |u0) (:text |:refer) (:type :leaf)
                    |r $ {} (:at 1634703941759) (:by |u0) (:type :expr)
                      :data $ {}
                        |T $ {} (:at 1634703941759) (:by |u0) (:text |calcit-dirname) (:type :leaf)
                        |j $ {} (:at 1634703953240) (:by |u0) (:text |calcit-filename) (:type :leaf)
        :proc $ {} (:at 1633149625774) (:by |u0) (:type :expr)
          :data $ {}
      |fswatch.util $ {}
        :configs $ {}
        :defs $ {}
          |get-dylib-ext $ {} (:at 1630231398718) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1630231418304) (:by |u0) (:text |defmacro) (:type :leaf)
              |j $ {} (:at 1633181058353) (:by |u0) (:text |get-dylib-ext) (:type :leaf)
              |r $ {} (:at 1630231398718) (:by |u0) (:type :expr)
                :data $ {}
              |v $ {} (:at 1630231403270) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1630231423910) (:by |u0) (:text |case-default) (:type :leaf)
                  |b $ {} (:at 1630231429893) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1630231433951) (:by |u0) (:text |&get-os) (:type :leaf)
                  |j $ {} (:at 1630231427453) (:by |u0) (:text "|\".so") (:type :leaf)
                  |r $ {} (:at 1630231437150) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1630231439152) (:by |u0) (:text |:macos) (:type :leaf)
                      |j $ {} (:at 1630231447585) (:by |u0) (:text "|\".dylib") (:type :leaf)
                  |v $ {} (:at 1630231448478) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1630231449901) (:by |u0) (:text |:windows) (:type :leaf)
                      |j $ {} (:at 1630231461388) (:by |u0) (:text "|\".dll") (:type :leaf)
          |get-dylib-path $ {} (:at 1634804142034) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1634804142034) (:by |u0) (:text |defn) (:type :leaf)
              |j $ {} (:at 1634804142034) (:by |u0) (:text |get-dylib-path) (:type :leaf)
              |n $ {} (:at 1634804146574) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1634804230294) (:by |u0) (:text |p) (:type :leaf)
              |r $ {} (:at 1634804145483) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1634804145483) (:by |u0) (:text |str) (:type :leaf)
                  |j $ {} (:at 1634804145483) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1634804145483) (:by |u0) (:text |or-current-path) (:type :leaf)
                      |j $ {} (:at 1634804145483) (:by |u0) (:text |calcit-dirname) (:type :leaf)
                  |r $ {} (:at 1634804157377) (:by |u0) (:text |p) (:type :leaf)
                  |v $ {} (:at 1634804145483) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1634804145483) (:by |u0) (:text |get-dylib-ext) (:type :leaf)
          |or-current-path $ {} (:at 1630245582276) (:by |u0) (:type :expr)
            :data $ {}
              |T $ {} (:at 1630245583936) (:by |u0) (:text |defn) (:type :leaf)
              |j $ {} (:at 1633181131099) (:by |u0) (:text |or-current-path) (:type :leaf)
              |r $ {} (:at 1630245582276) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1630245585364) (:by |u0) (:text |p) (:type :leaf)
              |v $ {} (:at 1630245585942) (:by |u0) (:type :expr)
                :data $ {}
                  |T $ {} (:at 1630245586336) (:by |u0) (:text |if) (:type :leaf)
                  |j $ {} (:at 1630245586894) (:by |u0) (:type :expr)
                    :data $ {}
                      |T $ {} (:at 1630245614560) (:by |u0) (:text |blank?) (:type :leaf)
                      |j $ {} (:at 1630245615061) (:by |u0) (:text |p) (:type :leaf)
                  |r $ {} (:at 1630245616843) (:by |u0) (:text "|\".") (:type :leaf)
                  |v $ {} (:at 1630245618366) (:by |u0) (:text |p) (:type :leaf)
        :ns $ {} (:at 1633181044360) (:by |u0) (:type :expr)
          :data $ {}
            |T $ {} (:at 1633181044360) (:by |u0) (:text |ns) (:type :leaf)
            |j $ {} (:at 1633181044360) (:by |u0) (:text |fswatch.util) (:type :leaf)
            |r $ {} (:at 1634804160546) (:by |u0) (:type :expr)
              :data $ {}
                |T $ {} (:at 1634804161330) (:by |u0) (:text |:require) (:type :leaf)
                |j $ {} (:at 1634804162771) (:by |u0) (:type :expr)
                  :data $ {}
                    |T $ {} (:at 1634804167270) (:by |u0) (:text |fswatch.$meta) (:type :leaf)
                    |j $ {} (:at 1634804168120) (:by |u0) (:text |:refer) (:type :leaf)
                    |r $ {} (:at 1634804168421) (:by |u0) (:type :expr)
                      :data $ {}
                        |T $ {} (:at 1634804171748) (:by |u0) (:text |calcit-dirname) (:type :leaf)
                        |j $ {} (:at 1634804175462) (:by |u0) (:text |calcit-filename) (:type :leaf)
        :proc $ {} (:at 1633181044360) (:by |u0) (:type :expr)
          :data $ {}
  :users $ {}
    |u0 $ {} (:avatar nil) (:id |u0) (:name |chen) (:nickname |chen) (:password |d41d8cd98f00b204e9800998ecf8427e) (:theme :star-trail)
