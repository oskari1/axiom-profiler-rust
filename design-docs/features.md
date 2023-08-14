Parser
- accepts Z3 4.12 trace files.
- can handle the following line cases:
    - `version-info`
    - `mk-quant`/`mk-lambda` 
    - `mk-var`
    - `mk-proof`/`mk-app`
    - `attach-meaning`
    - `attach-vars`
    - `attach-enode`
    - `eq-expl`
    - `new-match`
    - `inst-discovered`
    - `instance`
    - `end-of-instance`
    - `eof`
- *other line cases can be handled by creating a new implementation of Z3LogParser, or modifying the one(s) in the project*
- prints formatted representations of the collections for terms, quantifiers, instantiations, equality explanations, instantiation dependencies.
- outputs the instantiation graph in Dot format.
- calls Graphviz's `dot` program to render Dot output as an SVG.

Actix-Web server

Yew GUI