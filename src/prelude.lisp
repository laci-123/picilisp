(define t (quote t))

(define nil ())

(define defmacro (macro (name params body)
    (list (quote define) name (list (quote macro) params body))))

(defmacro defun (name params body)
  (list (quote define) name (list (quote lambda) params body)))

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

(defun last (things)
  (if things
      (if (cdr things)
          (last (cdr things))
          (car things))
      (signal (quote empty-list))))
