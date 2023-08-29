(define t (quote t))

(define nil ())

(define defmacro (macro (name params body)
    (list (quote define) name (list (quote macro) params body))))

(defmacro defun (name params body)
  (list (quote define) name (list (quote lambda) params body)))

(defmacro defspecial (name params body)
  (list (quote define) name (list (quote special-lambda) params body)))

(defun unzip-list (pairs)
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
  ((lambda (params-args)
     ((lambda (params args)
        (cons (list (quote lambda) params body) args))
      (car params-args)
      (cdr params-args)))
   (unzip-list bindings)))

(defun fold (f init things)
  (if things
      (f init (fold f (car things) (cdr things)))
      init))

(defun map (f things)
  (if things
      (cons (f (car things)) (map f (cdr things)))
      nil))

(defun last (things)
  (if things
      (if (cdr things)
          (last (cdr things))
          (car things))
      (signal (quote empty-list))))

(defmacro block (& body)
  (list (quote last) (cons (quote list) body)))

(defmacro and (x y)
  (list (quote if) x y nil))

(defmacro or (x y)
  (list (quote if) x x y))

(defmacro not (x)
  (list (quote if) x nil t))

(defun substract (x y)
  (add x (multiply -1 y)))

(defun + (& numbers)
  (fold add 0 numbers))

(defun * (& numbers)
  (fold multiply 1 numbers))

(defun - (& numbers)
  (if numbers
      (let (first (car numbers)
            rest  (cdr numbers))
        (if rest
            (substract first (fold add 0 rest))
            (multiply -1 first)))
      0))

(defun / (& numbers)
  (if numbers
      (let (first (car numbers)
            rest  (cdr numbers))
        (if rest
            (divide first (fold multiply 1 rest))
            (divide 1 first)))
      1))

(defun get-property (key plist)
  (if plist
      (let (first  (car plist)
            second (car (cdr plist))
            rest   (cdr (cdr plist)))
        (if (= key first)
            second
            (get-property key rest)))
      nil))
