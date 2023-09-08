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

(defun fold (f init things)
  "Return the result of applying `f` to `init` and the first element of `things`,
then applying `f` to that result and the third element of `things` etc.
If `things` is empty then just return `init`."
  (if things
      (f init (fold f (car things) (cdr things)))
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
  "Return the last element of `things`"
  (if things
      (if (cdr things)
          (last (cdr things))
          (car things))
      (signal (quote empty-list))))

(defmacro block (& body)
  "Execute all forms in `body` then return the result of the last one."
  (if body
      (let (params (map (lambda (_) (gensym)) body))
        (cons (list (quote lambda) params (last params)) body))
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
  (fold add 0 numbers))

(defun * (& numbers)
  "Multiply all numbers. Return 1 if called with 0 arguments."
  (fold multiply 1 numbers))

(defun - (& numbers)
  "If called with 0 arguments: return 1.
If called with 1 argument: negate it.
Otherwise substract all but the first argument from the first one."
  (if numbers
      (let (first (car numbers)
            rest  (cdr numbers))
        (if rest
            (substract first (fold add 0 rest))
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
            (divide first (fold multiply 1 rest))
            (divide 1 first)))
      1))

(defun append (list1 list2)
  "Append `list1` to the beginning of `list2`."
  (if list1
      (cons (car list1)
            (append (cdr list1) list2))
      list2))
