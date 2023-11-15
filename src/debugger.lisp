(export '(debug-eval))


(defun lookup (key env env-module)
  ""
  (if env
      (let (key-value (car env))
        (if (= key (car key-value))
            (cdr key-value)
            (lookup key (cdr env) env-module)))
      (with-current-module key env-module)))

(defun add-parameters (params args env)
  ""
  (if params
      (let (first-param (car params)
            rest-params (cdr params)
            first-arg   (car args)
            rest-args   (cdr args))
        (if (= first-param '&)
            (cons (cons (car rest-params) args)
                  env)
            (cons (cons first-param first-arg)
                  (add-parameters rest-params rest-args env))))
      env))

(defun highlight-list-elem (elems n)
  ""
  (concat "("
          (let (len (length elems))
            (apply concat 
                   (map (lambda (xi)
                          (let (x (car xi), i (cdr xi))
                            (concat (if (= i n)
                                        (concat "「" (print x) "」")
                                        (print x))
                                    (if (/= i (substract len 1))
                                        " "
                                        ""))))
                        (zip elems (range len)))))
          ")"))

(defun debug-list (expr env env-module step-in)
  ""
  (let (operator (car expr)
        operands (cdr expr)
        highlight-and-debug (lambda (x i)           
                              (if step-in
                                  (block
                                    (send (list 'kind 'HIGHLIGHT-ELEM, 'string (highlight-list-elem expr i)))
                                    (debug-eval-internal x env env-module (= (. (receive) 'command) 'STEP-IN)))
                                  (debug-eval-internal x env env-module nil))))
    (case
      ((= operator 'quote)  (block
                              (when step-in
                                (send (list 'kind 'HIGHLIGHT-ELEM, 'string (highlight-list-elem expr 1))))
                              (car operands)))
      ((= operator 'if)     (let (condition (car operands)
                                  then      (car (cdr operands))
                                  otherwise (car (cdr (cdr operands))))
                              (if (highlight-and-debug condition 1)
                                  (highlight-and-debug then      2)
                                  (highlight-and-debug otherwise 3))))
      ((= operator 'eval)   (let (operand (highlight-and-debug (car operands) 1))
                              (highlight-and-debug operand 1)))
      ((= operator 'trap)   (make-trap (car operands) (car (cdr operands))) env)
      ((= operator 'lambda) (make-function (car operands)
                                           (car (cdr operands))
                                           env
                                           env-module
                                           'lambda-type))
      ('otherwise           (let (evaled-expr (map (lambda (xi) (highlight-and-debug (car xi) (cdr xi)))
                                                   (enumerate expr)))
                              (let (body (get-body (car evaled-expr)))
                                (block
                                  (when step-in
                                    (block
                                      (receive)
                                      (send (list 'kind 'ALL-ELEMS-EVALED, 'expression (print expr), 'result (print evaled-expr)))))
                                  (if body
                                      (debug-eval-internal (car body)
                                                           (add-parameters (get-parameters (car evaled-expr))
                                                                           (cdr evaled-expr)
                                                                           (get-environment (car evaled-expr)))
                                                           (get-environment-module (car evaled-expr))
                                                           step-in)
                                      (call-native-function (car evaled-expr) (cdr evaled-expr) env)))))))))
                                      
(defun debug-eval-internal (expr env env-module step-in)
  ""
  (eval (trap
         (block
           (when step-in 
             (send (list 'kind 'EVAL, 'string (print expr))))
           (let (result (let (type (type-of expr))
                          (case
                            ((= type 'list-type)   (debug-list expr env env-module step-in))
                            ((= type 'cons-type)   (cons (debug-eval-internal (car expr) env env-module step-in)
                                                         (debug-eval-internal (cdr expr) env env-module step-in)))
                            ((= type 'symbol-type) (lookup expr env env-module))
                            ((= type 'trap-type)   (let (nt (destructure-trap expr))
                                                     (let (normal-body (car nt)
                                                           trap-body   (car (cdr nt)))
                                                       (eval (trap
                                                              (debug-eval-internal normal-body env env-module step-in)
                                                              (block
                                                                (receive)
                                                                (send (list 'kind 'SIGNAL-TRAPPED, 'string (print *trapped-signal*)))
                                                                (debug-eval-internal trap-body (cons (cons '*trapped-signal* *trapped-signal*) env) env-module step-in)))))))
                            ('otherwise            expr))))
             (block
               (when step-in
                 (block
                   (receive)
                   (send (list 'kind 'RETURN-VALUE, 'expression (print expr), 'result (print result)))
                   (receive)
                   (send (list 'kind 'RETURN))))
               result)))
         (block
           (when step-in
             (block
               (receive)
               (send (list 'kind 'SIGNAL-UNWIND, 'string (print *trapped-signal*)))))
           (signal *trapped-signal*)))))

(defun sequence-changed (elems)
  ""
  (foldr (lambda (x xs) (if (. x 'changed)
                            (list 'result (cons (. x 'result) (. xs 'result)), 'changed t)
                            (list 'result (cons (. x 'result) (. xs 'result)), 'changed (. xs 'changed))))
         (list 'result nil, 'changed nil)
         elems))

(defun debug-expand-list (expr env env-module step-in)
  ""
  (let (operator (car expr)
        operands (cdr expr))
    (case
      ((= operator 'quote) (list 'result  expr
                                 'changed nil))
      ((= operator 'macro) (list 'result  (make-function (car operands) (car (cdr operands)) env env-module 'macro-type)
                                 'changed nil))
      ('otherwise           (let (expanded-operator-rc (debug-expand operator env env-module step-in)
                                  expanded-operands-rc (sequence-changed (map (lambda (x) (debug-expand x env env-module step-in)) operands)))
                              (let (expanded-operator (. expanded-operator-rc 'result)
                                    expanded-operands (. expanded-operands-rc 'result) 
                                    changed           (or (. expanded-operator-rc 'changed) (. expanded-operands-rc 'changed)))
                                (if (= 'macro (. (get-metadata expanded-operator) 'function-kind))
                                    (let (body (get-body expanded-operator))
                                      (list 'result (if body
                                                        (debug-eval-internal (car body)
                                                                             (add-parameters (get-parameters expanded-operator)
                                                                                             expanded-operands
                                                                                             (get-environment expanded-operator))
                                                                             (get-environment-module expanded-operator)
                                                                             step-in)
                                                        (call-native-function expanded-operator expanded-operands env))
                                            'changed t))
                                    (list 'result  (cons expanded-operator expanded-operands)
                                          'changed changed))))))))
         
(defun debug-expand (expr env env-module step-in)
  ""
  (let (type (type-of expr))
    (case
      ((= type 'list-type)   (debug-expand-list expr env env-module step-in)) 
      ((= type 'cons-type)   (let (expanded-car (debug-expand (car expr) env env-module step-in)
                                                expanded-cdr (debug-expand (cdr expr) env env-module step-in))
                               (let (changed (or (. expanded-car 'changed) (. expanded-cdr 'changed)))
                                 (list 'result  (cons (. expanded-car 'result) (. expanded-cdr 'result))
                                       'changed changed))))
      ((= type 'symbol-type) (eval (trap
                                    (let (expanded (lookup expr env env-module))
                                      (if (= 'macro (. (get-metadata expanded) 'function-kind))
                                          (list 'result expanded, 'changed t)
                                          (list 'result expr,     'changed nil)))
                                    (if (= (. *trapped-signal* 'kind) 'unbound-symbol)
                                        (list 'result expr,     'changed nil)
                                        (signal *trapped-signal*)))))
      ('otherwise            (list 'result  expr
                                   'changed nil)))))

(defun keep-expanding (expr env env-module step-in ch)
  ""
  (block
    (when step-in
      (send (list 'kind 'BEGIN-EXPANDING, 'string (print expr))))
    (let (e (debug-expand expr env env-module step-in))
      (let (changed  (. e 'changed)
            expanded (. e 'result))
        (if changed
            (keep-expanding expanded env env-module step-in t)
            (list 'result expanded, 'changed ch))))))
  
(defun debug-eval (expr env step-into-macroexpand)
  "Evaluate `expr` using `env` as the local environment while emitting and listening to debugger messages.
If `step-into-macroexpand` is non-nil then also emit debugger messages in the macro expansion phase."
  (let (e (keep-expanding expr env (get-current-module) step-into-macroexpand nil))
    (let (changed  (. e 'changed)
          expanded (. e 'result))
      (block
        (when changed
          (send (list 'kind 'EXPAND, 'expression (print expr), 'expanded (print expanded))))
        (debug-eval-internal expanded env (get-current-module) t)))))
