(define t (quote t))

(define nil ())

(define defmacro (macro (name params body)
    (list (quote define) name (list (quote macro) params body))))

(defmacro defun (name params body)
  (list (quote define) name (list (quote lambda) params body)))

(defun unzip-lists (lists)
  (if lists
      ((lambda (fsts-snds)
         (cons
          (cons (car      (car lists))  (car fsts-snds))
          (cons (car (cdr (car lists))) (cdr fsts-snds))))
       (unzip-lists (cdr lists)))
      (cons nil nil)))

(defmacro let (bindings body)
  ((lambda (params-args)
     ((lambda (params args)
        (cons (list (quote lambda) params body) args))
      (car params-args)
      (cdr params-args)))
   (unzip-lists bindings)))
