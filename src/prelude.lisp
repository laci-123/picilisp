(define t (quote t) "`t` is the canonical true value.")

(define nil () "`nil` is a synonym for the empty list `()`.")

(define defmacro (macro (name params doc-string body)
    (list (quote define) name (list (quote macro) params body) doc-string)) "Globally define `name` as a macro.")

(defmacro defun (name params doc-string body)
  "Globally define `name` as a lambda function."
  (list (quote define) name (list (quote lambda) params body) doc-string))

(defmacro defspecial (name params doc-string body)
  "Globally define `name` as a special-lambda function."
  (list (quote define) name (list (quote special-lambda) params body) doc-string))

(defspecial if (condition then otherwise)
  "Evaluate `then` if and only if `condition` evaluates to non-nil,
and evaluate `otherwise` if and only if `condition` evaluates to nil."
  (eval (branch (eval condition) then otherwise)))

(defun unzip-list (pairs)
  "Group the odd and even numbered elements of `pairs` into two separate lists."
  (if pairs
      ((lambda (fsts-snds)
         (cons
          (cons (car      pairs)  (car fsts-snds))
          (cons (car (cdr pairs)) (cdr fsts-snds))))
       (unzip-list (cdr (if (cdr pairs)
                            (cdr pairs)
                            (signal (quote odd-number-of-elements))))))
      (cons nil nil)))

(defmacro let (bindings body)
"Bind variables according to `bindings` then eval `body`.
The value of the last form in `body` is returned."
  ((lambda (params-args)
     ((lambda (params args)
        (cons (list (quote lambda) params body) args))
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
  (list (list (quote unrest) f) args-list))

(defun last (things)
  "Return the last element of `things`."
  (if things
      (if (cdr things)
          (last (cdr things))
          (car things))
      (signal (quote empty-list))))

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
          (cons (list (quote lambda) params end) init-body)))
      nil))

(defmacro and (x y)
  "Logical and."
  (list (quote if) x y nil))

(defmacro or (x y)
  "Logical or."
  (list (quote if) x x y))

(defmacro not (x)
  "Logical not."
  (list (quote if) x nil t))

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
        (concat (if (= (type-of thing) (quote function))
                    (concat "("
                            (print (get-property (quote function-kind) metadata))
                            " "
                            (print (get-property (quote parameters) metadata))
                            " ...)\n\n")
                    "")
                (or (get-property (quote documentation) metadata)
                    "[No documentation]")
                "\n\nDefined in:\n "
                (let (source (get-property (quote file) metadata))
                  (if (= source (quote native))
                      "Rust source."
                      (concat (print source)
                              ":"
                              (print (get-property (quote line) metadata))
                              ":"
                              (print (get-property (quote column) metadata))))))
        "No description available")))

(defmacro case (& cases)
  "Each element of `cases` should be in the following form: `(condition value)`.
Return the `value` of the first element whose `condition` evaluates to true.
If non of them is true then return `nil`."
  (foldr (lambda (c acc) (let (condition (car c), value (car (cdr c)))
                           (list (quote if) condition value acc)))
        nil
        cases)) 

(defmacro catch (kind body)
  "Catches any signal whose `kind` property is equal to `kind`.
Meant to be used as part of the `try` macro.
`body` should be a lambda with one parameter. This parameters will be set to the caught signal."
  (list (quote test)
        (list (quote =)
              (list (quote get-property)
                    (list (quote quote)
                          (quote kind))
                    (quote *trapped-signal*))
              (list (quote quote) kind))
        (quote body)
        body))

(defmacro catch-all (body)
  "Catches any signal.
Meant to be used as part of the `try` macro.
`body` should be a lambda with one parameter. This parameters will be set to the caught signal."
  (list (quote test) t
        (quote body) body))

(defmacro try (body & catchers)
  "Try to evaluate `body`.
If a signal is emitted while evaluating `body`, evaluate the first catcher in `catchers`
that catches the signal."
  (list (quote eval)
         (list (quote trap)
               body
               (cons (quote case)
                     (map (lambda (catcher) (list (get-property (quote test) catcher)
                                                  (list (get-property (quote body) catcher) (quote *trapped-signal*))))
                          catchers)))))

(defmacro throw (& body)
  "Emit a signal that is a property-list made of the key-value pairs in `body`."
  (list (quote signal)
        (cons (quote list)
              body)))

(defun repl (prompt initial-input)
  "(R)ead an expression from standard input,
(E)valuated it,
(P)rint the result to standard output,
then repeat (or (L)oop) from the beginning.
Stop the loop when end of input (EOF) is reached."
  (try
   (let (current-input (concat initial-input (input prompt)))
     (let (read-result (read current-input))
       (let (read-status (get-property (quote status) read-result))
         (case ((= read-status (quote invalid))    (throw (quote kind) (quote invalid-string), (quote source) (quote repl)))
               ((= read-status (quote nothing))    (repl prompt nil))
               ((= read-status (quote incomplete)) (repl "... " current-input))
               ((= read-status (quote error))      (throw (quote kind) (quote syntax-error), (quote source) (quote repl), (quote details) (get-property (quote error) read-result)))
               ((= read-status (quote ok))         (block (output (print (eval (get-property (quote result) read-result))))
                                                          (repl ">>> " nil)))
               (t                                  (throw (quote kind) (quote unknown-read-status), (quote source) (qoute repl), (quote read-status) read-status))))))
   (catch eof
     (lambda (_) (block (output "")
                        (quote ok))))
   (catch-all
    (lambda (error) (block (output (concat "ERROR: " (print error)))
                           (repl ">>> " nil))))))
