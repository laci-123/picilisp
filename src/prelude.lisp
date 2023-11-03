(define 't 't "`t` is the canonical true value.")

(define 'nil () "`nil` is a synonym for the empty list `()`.")

(define 'defmacro (macro (name params doc-string body)
    (list 'define (list 'quote name) (list 'macro params body) doc-string)) "Globally define `name` as a macro.")

(defmacro defun (name params doc-string body)
  "Globally define `name` as a lambda function."
  (list 'define (list 'quote name) (list 'lambda params body) doc-string))

(defun unzip-list (pairs)
  "Group the odd and even numbered elements of `pairs` into two separate lists."
  (if pairs
      ((lambda (fsts-snds)
         (cons
          (cons (car      pairs)  (car fsts-snds))
          (cons (car (cdr pairs)) (cdr fsts-snds))))
       (unzip-list (cdr (if (cdr pairs)
                            (cdr pairs)
                            (signal 'odd-number-of-elements)))))
      (cons nil nil)))

(defmacro let (bindings body)
"Bind variables according to `bindings` then eval `body`.
The value of the last form in `body` is returned."
  ((lambda (params-args)
     ((lambda (params args)
        (cons (list 'lambda params body) args))
      (car params-args)
      (cdr params-args)))
   (unzip-list bindings)))

(defun foldl (f init things)
  "Return the result of applying `f` to `init` and the first element of `things`,
then applying `f` to that result and the second element of `things` and so on.
If `things` is empty then just return `init`."
  (if things
      (foldl f (f init (car things)) (cdr things))
      init))

(defun foldr (f init things)
  "Return the result of applying `f` to `init` and the last element of `things`,
then applying `f` to that result and the second to last element of `things` and so on.
If `things` is empty then just return `init`."
  (if things
      (f (car things) (foldr f init (cdr things)))
      init))

(defun map (f things)
  "Apply `f` to each element of `things`, and make a list of the results."
  (if things
      (cons (f (car things)) (map f (cdr things)))
      nil))

(defmacro apply (f args-list)
  "Apply `f` to `args-list`, as if each element of `args-list` were a parameter of `f`."
  (list (list 'unrest f) args-list))

(defun last (things)
  "Return the last element of `things`."
  (if things
      (if (cdr things)
          (last (cdr things))
          (car things))
      (signal 'empty-list)))

(defun init (things)
  "Return all elements of `things` except the last one."
  (if things
      (if (cdr things)
          (cons (car things) (init (cdr things)))
          nil)
      nil))

(defmacro block (& body)
  "Execute all forms in `body` then return the result of the last one."
  (if body
      (let (init-body (init body))
        (let (params (map (lambda (_) (gensym)) init-body)
              end    (last body))
          (cons (list 'lambda params end) init-body)))
      nil))

(defmacro and (x y)
  "Logical and."
  (list 'if x y nil))

(defmacro or (x y)
  "Logical or."
  (list 'if x x y))

(defmacro not (x)
  "Logical not."
  (list 'if x nil t))

(defun /= (x y)
  "Not equals"
  (not (= x y)))

(defun + (& numbers)
  "Add all numbers together. Return 0 if called whith 0 arguments."
  (foldl add 0 numbers))

(defun * (& numbers)
  "Multiply all numbers. Return 1 if called with 0 arguments."
  (foldl multiply 1 numbers))

(defun - (& numbers)
  "If called with 0 arguments: return 1.
If called with 1 argument: negate it.
Otherwise substract all but the first argument from the first one."
  (if numbers
      (let (first (car numbers)
            rest  (cdr numbers))
        (if rest
            (substract first (foldl add 0 rest))
            (multiply -1 first)))
      0))

(defun / (& numbers)
  "If called with 0 arguments: return 1.
If called with 1 argument: return 1 divided by that arguments.
Otherwise substract all but the first argument from the first one."
  (if numbers
      (let (first (car numbers)
            rest  (cdr numbers))
        (if rest
            (divide first (foldl multiply 1 rest))
            (divide 1 first)))
      1))

(defun append (list1 list2)
  "Append `list1` to the beginning of `list2`."
  (if list1
      (cons (car list1)
            (append (cdr list1) list2))
      list2))

(defun concat (& lists)
  "Concatenate all lists in `lists`."
  (let (f (lambda (f xs) (if xs
                             (append (car xs) (f f (cdr xs)))
                             nil)))
    (f f lists)))

(defun describe (thing)
  "Print all available metadata about `thing` in a human-readable format."
  (let (metadata (get-metadata thing))
    (if metadata
        (concat (if (= (type-of thing) 'function-type)
                    (concat "("
                            (print (get-property 'function-kind metadata))
                            " "
                            (print (get-property 'parameters metadata))
                            " ...)\n\n")
                    "")
                (or (get-property 'documentation metadata)
                    "[No documentation]")
                "\n\nDefined in:\n "
                (let (source (get-property 'file metadata))
                  (if (= source 'native)
                      "Rust source."
                      (concat (print source)
                              ":"
                              (print (get-property 'line metadata))
                              ":"
                              (print (get-property 'column metadata))))))
        "No description available")))

(defmacro case (& cases)
  "Each element of `cases` should be in the following form: `(condition value)`.
Return the `value` of the first element whose `condition` evaluates to true.
If non of them is true then return `nil`."
  (foldr (lambda (c acc) (let (condition (car c), value (car (cdr c)))
                           (list 'if condition value acc)))
        nil
        cases)) 

(defun get-property-safe (key plist )
  "Same as `get-property`, but return nil if either `key` is not found in `plist` or if `plist` is not a property-list"
  (eval (trap
   (get-property key plist)
   nil)))

(defmacro catch (kind body)
  "Catches any signal whose `kind` property is equal to `kind`.
Meant to be used as part of the `try` macro.
`body` should be a lambda with one parameter. This parameters will be set to the caught signal."
  (list 'test
        (list '=
              (list 'get-property-safe
                    (list 'quote
                          'kind)
                    '*trapped-signal*)
              (list 'quote kind))
        'body
        body))

(defmacro catch-all (body)
  "Catches any signal.
Meant to be used as part of the `try` macro.
`body` should be a lambda with one parameter. This parameters will be set to the caught signal."
  (list 'test t
        'body body))

(defmacro try (body & catchers)
  "Try to evaluate `body`.
If a signal is emitted while evaluating `body`, evaluate the first catcher in `catchers`
that catches the signal."
  (list 'eval
         (list 'trap
               body
               (cons 'case
                     (map (lambda (catcher) (list (get-property 'test catcher)
                                                  (list (get-property 'body catcher) '*trapped-signal*)))
                          catchers)))))

(defmacro throw (& body)
  "Emit a signal that is a property-list made of the key-value pairs in `body`."
  (list 'signal
        (cons 'list
              body)))

(defun pretty-print-error (error)
  "Print `erorr` in a more human-readable format.
If `error` is not a valid property-list then just simply print it using the `print` function."
  (if error
      (try
       (let (key (car error), value (car (cdr error)))
         (concat (print key)
                 ":\n"
                 (if (= key 'symbol)
                     (let (md (get-metadata value))
                       (concat (print value)
                               "\n at: "
                               (print (get-property 'file md))
                               (if (get-property 'line md)
                                   (concat ":" (print (get-property 'line md)) ":" (print (get-property 'column md)))
                                   nil)))
                     (print value))
                 "\n\n"
                 (pretty-print-error (cdr (cdr error)))))
       (catch-all (lambda (_) (print error))))
      ""))

(defun repl (prompt initial-input)
  "(R)ead an expression from standard input,
(E)valuated it,
(P)rint the result to standard output,
then repeat (or (L)oop) from the beginning.
Stop the loop when end of input (EOF) is reached."
  (try
   (let (current-input (concat initial-input (input prompt)))
     (let (read-result (read current-input))
       (let (read-status (get-property 'status read-result))
         (case ((= read-status 'invalid)    (throw 'kind 'invalid-string, 'source 'repl))
               ((= read-status 'nothing)    (repl prompt nil))
               ((= read-status 'incomplete) (repl "... " current-input))
               ((= read-status 'error)      (throw 'kind 'syntax-error, 'source 'repl, 'details (get-property 'error read-result)))
               ((= read-status 'ok)         (block (output (print (eval (get-property 'result read-result))))
                                                          (repl ">>> " nil)))
               (t                                  (throw 'kind 'unknown-read-status, 'source (qoute repl), 'read-status read-status))))))
   (catch eof
     (lambda (_) (block (output "")
                        'ok)))
   (catch-all
    (lambda (error) (block (output (concat "UNHANDLED ERROR:\n\n" (pretty-print-error error)))
                           (repl ">>> " nil))))))

(defun read-eval-print (string)
  "Read a string, evaluate it then print it into a string.
If a signal is emmited during read evaluate or print then pretty-print it then forward it."
  (try
   (let (read-result (read string))
     (let (read-status (get-property 'status read-result))
       (case ((= read-status 'invalid)    (throw 'kind 'invalid-string, 'source 'repl))
             ((= read-status 'nothing)    "")
             ((= read-status 'incomplete) (throw 'kind 'syntax-error,   'source 'read-eval-print, 'details 'incomplete-input))
             ((= read-status 'error)      (throw 'kind 'syntax-error,   'source 'repl,            'details (get-property 'error read-result)))
             ((= read-status 'ok)         (print (eval (get-property 'result read-result))))
             (t                           (throw 'kind 'unknown-read-status, 'source (qoute repl), 'read-status read-status)))))
  (catch-all
   (lambda (error) (signal (pretty-print-error error))))))

(defun loop (step condition body)
  "Call the one-parameter function `condition` on the result of zero-parameter function `step`.
If it evaluates non-nil, then evaluate body and repeat, otherwise exit the loop."
  (let (x (step))
    (if (condition x)
        (block
          (body x)
          (loop step condition body))
        nil)))

(defmacro while-let (binding-step condition body)
  "Usage example: (while-let (x (input \"...\")) (/= x \"QUIT\") (output x))"
  (let (binding (car binding-step)
        step    (car (cdr binding-step)))
    (list 'loop
          (list 'lambda () step)
          (list 'lambda (list binding) condition)
          (list 'lambda (list binding) body))))

(defun infinite-loop (x)
  "for testing purposes"
  (block
    (output (print x))
    (infinite-loop (add x 1))))

(defun lookup (key env)
  ""
  (if env
      (let (key-value (car env))
        (if (= key (car key-value))
            (cdr key-value)
            (lookup key (cdr env))))
      (eval key)))

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

(defun debug-list (expr env)
  ""
  (let (operator (car expr)
        operands (cdr expr))
    (case
      ((= operator 'quote)  (car operands))
      ((= operator 'if)     (let (condition (car operands)
                                  then      (car (cdr operands))
                                  otherwise (car (cdr (cdr operands))))
                              (if (debug-eval-internal condition env)
                                  (debug-eval-internal then      env)
                                  (debug-eval-internal otherwise env))))
      ((= operator 'eval)   (debug-eval-internal (debug-eval-internal (car operands) env) env))
      ((= operator 'trap)   (make-trap (car operands) (car (cdr operands))) env)
      ((= operator 'lambda) (make-function (car operands)
                                           (car (cdr operands))
                                           env
                                           'lambda-type))
      ('otherwise           (let (evaled-operator (debug-eval-internal operator env)
                                  evaled-operands (map (lambda (x) (debug-eval-internal x env)) operands))
                              (let (body (get-body evaled-operator))
                                (if body
                                    (debug-eval-internal (car body)
                                                (add-parameters (get-parameters evaled-operator)
                                                                evaled-operands
                                                                (get-environment evaled-operator)))
                                    (call-native-function evaled-operator evaled-operands env))))))))
                                      
(defun debug-eval-internal (expr env)
  ""
  (block
    (output (concat "# EVAL:   " (print expr) " ENV: " (print env)))
    (let (result (let (type (type-of expr))
                   (case
                     ((= type 'list-type)   (debug-list expr env))
                     ((= type 'cons-type)   (cons (debug-eval-internal (car expr) env)
                                                  (debug-eval-internal (cdr expr) env)))
                     ((= type 'symbol-type) (lookup expr env))
                     ((= type 'trap-type)   (let (nt (destructure-trap expr))
                                              (let (normal-body (car nt)
                                                    trap-body   (car (cdr nt)))
                                                (eval (trap
                                                       (debug-eval-internal normal-body env)
                                                       (block
                                                         (output (concat "# SIGNAL TRAPPED: " (print *trapped-signal*)))
                                                         (debug-eval-internal trap-body (cons (cons '*trapped-signal* *trapped-signal*) env))))))))
                     ('otherwise            expr))))
      (block
        (output (concat "# RETURN: " (print expr) " --> " (print result)))
        result))))

(defun debug-expand-list (expr env)
  ""
  (let (operator (car expr)
        operands (cdr expr))
    (case
      ((= operator 'quote) (list 'result  expr
                                 'changed nil))
      ((= operator 'macro) (list 'result  (make-function (car operands) (car (cdr operands)) env 'macro-type)
                                 'changed nil))
      ('otherwise           (let (expanded-operator (get-property 'result (debug-expand operator env))
                                  expanded-operands (map (lambda (x) (get-property 'result (debug-expand x env))) operands))
                              (if (= 'macro (get-property 'function-kind (get-metadata expanded-operator)))
                                  (let (body (get-body expanded-operator))
                                    (list 'result (if body
                                                      (debug-eval-internal (car body)
                                                                           (add-parameters (get-parameters expanded-operator)
                                                                                           expanded-operands
                                                                                           (get-environment expanded-operator)))
                                                      (call-native-function expanded-operator expanded-operands env))
                                          'changed t))
                                  (list 'result  (cons expanded-operator expanded-operands)
                                        'changed nil)))))))
         
(defun debug-expand (expr env)
  ""
  (let (type (type-of expr))
    (case
      ((= type 'list-type)   (debug-expand-list expr env)) 
      ((= type 'cons-type)   (let (expanded-car (debug-expand (car expr) nil)
                                   expanded-cdr (debug-expand (cdr expr) nil))
                               (let (changed (or (get-property 'changed expanded-car) (get-property 'changed expanded-cdr)))
                                 (list 'result  (cons (get-property 'result expanded-car) (get-property 'result expanded-cdr))
                                       'changed changed))))
      ((= type 'symbol-type) (eval (trap
                                    (let (expanded (lookup expr env))
                                      (if (= 'macro (get-property 'function-kind (get-metadata expanded)))
                                          (list 'result expanded, 'changed t)
                                          (list 'result expr,     'changed nil)))
                                    (if (= (get-property 'kind *trapped-signal*) 'unbound-symbol)
                                        (list 'result expr,     'changed nil)
                                        (signal *trapped-signal*)))))
      ('otherwise            (list 'result  expr
                                   'changed nil)))))

(defun keep-expanding (expr env ch)
  ""
  (let (e (debug-expand expr env))
    (let (changed  (get-property 'changed e)
          expanded (get-property 'result e))
      (if changed
          (keep-expanding expanded env t)
          (list 'result expanded, 'changed ch)))))
  
                                                
(defun debug-eval (expr env)
  ""
  (let (e (keep-expanding expr env nil))
    (let (changed  (get-property 'changed e)
          expanded (get-property 'result e))
      (block
        (if changed
            (output (concat "# EXPAND: " (print expr) " --> " (print expanded)))
            nil)
        (debug-eval-internal expanded env)))))
