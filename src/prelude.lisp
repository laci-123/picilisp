(export '(t nil *stdin* *stdout* defmacro defun unzip-list let when output input foldl foldr
          reverse zip length enumerate map apply last init block and or not /= <= >= + - * /
          range append concat describe case catch catch-all try throw get-property-safe
          load read-simple infinite-loop))


(define 't 't "`t` is the canonical true value.")

(define 'nil () "`nil` is a synonym for the empty list `()`.")

(define '*stdin* '*stdin* "standard input")

(define '*stdout* '*stdout* "standard output")

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
                            (signal (list 'kind 'wrong-argument, 'source 'unzip-list, 'details 'odd-number-of-elements))))))
      (cons nil nil)))

(defmacro let (bindings body)
"Bind variables according to `bindings` then eval `body`.
The value of the last form in `body` is returned."
  ((lambda (params-args)
     ((lambda (params args)
        (cons (list 'lambda params body) args))
      (car params-args)
      (cdr params-args)))
   (eval (trap (unzip-list bindings)
               (signal (list 'kind 'wrong-argument, 'source 'let, 'details 'incomplete-binding))))))

(defmacro when (condition then)
  "Same as `if` but the `otherwise` arm is always `nil`."
  (list 'if
        condition
        then
        nil))

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

(defun reverse (things)
  "Reverse the order of elements in `things`."
  (foldl (lambda (xs x) (cons x xs)) nil things))

(defun -zip (things1 things2 init)
  ""
  (if things1
      (if things2
          (-zip (cdr things1) (cdr things2) (cons (cons (car things1)
                                                   (car things2))
                                             init))
          init)
      init))

(defun zip (things1 things2)
  "Group the head of each list, followed by the second element of
each list, and so on. The number of returned groupings is equal
to the length of the shortest input list."
  (reverse (-zip things1 things2 nil)))

(defun -length (things n)
  ""
  (if things
      (-length (cdr things) (add n 1))
      n))

(defun length (things)
  "Return the number of elements in `things`."
  (-length things 0))

(defun enumerate (things)
  "Zip each element of `things` with its index (starting from 0)."
  (zip things (range (length things))))


(defun -map (f things init)
  ""
  (if things
      (-map f (cdr things) (cons (f (car things)) init))
      init))

(defun map (f things)
  "Apply `f` to each element of `things`, and make a list of the results."
  (reverse (-map f things nil)))

(defmacro apply (f args-list)
  "Apply `f` to `args-list`, as if each element of `args-list` were a parameter of `f`."
  (list (list 'unrest f) args-list))

(defun last (things)
  "Return the last element of `things`."
  (if things
      (if (cdr things)
          (last (cdr things))
          (car things))
      (signal (list 'kind 'wrong-argument, 'soruce 'last, 'details 'empty-list))))

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

(defun output (msg)
  "Write `msg` to stdout."
  (output-file *stdout* (concat msg "\n")))

(defun input (prompt)
  "Write `prompt` to stdout then read a line from stdin."
  (block
    (output-file *stdout* prompt)
    (input-file *stdin*)))

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

(defun <= (x y)
  "`x` is less than or equal to `y`"
  (or (< x y) (= x y)))

(defun >= (x y)
  "`x` is greater than or equal to `y`"
  (or (> x y) (= x y)))

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

(defun -range (n init)
  ""
  (if (= n -1)
      init
      (-range (substract n 1) (cons n init))))

(defun range (n)
  "Range of numbers from 0 to `n` (including 0, excluding `n`)."
  (-range (substract n 1) nil))

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
                    (let (thing-parts (destructure-function thing))
                      (concat "("
                              (print (. thing-parts 'kind))
                              " "
                              (print (. thing-parts 'parameters))
                              " ...)\n\n"))
                    "")
                (or (. metadata 'documentation)
                    "[No documentation]")
                "\n\nDefined in:\n "
                (let (source (. metadata 'file))
                  (if (= source 'native)
                      "Rust source."
                      (concat (print source)
                              ":"
                              (print (. metadata 'line))
                              ":"
                              (print (. metadata 'column))))))
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
  "Same as `.`, but return nil if either `key` is not found in `plist` or if `plist` is not a property-list"
  (eval (trap
   (. plist key)
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
                     (map (lambda (catcher) (list (. catcher 'test)
                                                  (list (. catcher 'body) '*trapped-signal*)))
                          catchers)))))

(defmacro throw (& body)
  "Emit a signal that is a property-list made of the key-value pairs in `body`."
  (list 'signal
        (cons 'list
              body)))

(defun --remove-extension (path)
  ""
  (when path
    (if (= (car path) %.)
        (reverse (cdr path))
        (--remove-extension (cdr path)))))

(defun -remove-extension (path)
  ""
  (or (--remove-extension (reverse path)) path))

(defun load (path)
  "Load the lisp module at the file path `path`."
  (load-all (input-file path) (-remove-extension path)))

(defun read-simple (input)
  "Like `read`, but simple return the result without any additional data.
Error if `read` would return non-ok status."
  (let (read-result (read input 'stdin 1 1))
    (if (= (. read-result 'status) 'ok)
        (. read-result 'result)
        (throw 'kind 'read-error, 'details read-result))))

(defun infinite-loop (x)
  "for testing purposes"
  (block
    (output (print x))
    (infinite-loop (add x 1))))
