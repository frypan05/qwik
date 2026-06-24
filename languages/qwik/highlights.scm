; Qwik lifecycle hooks — highlight as functions
((call_expression
  function: (identifier) @function
  (#match? @function "^(component|useTask|useVisibleTask|useSignal|useStore|useResource|useContext|useContextProvider|createContextId|useStyles|useStylesScoped|server|routeLoader|routeAction)\\$")))

; $ suffix marker — highlight as operator
((identifier) @operator
  (#match? @operator "\\$$"))