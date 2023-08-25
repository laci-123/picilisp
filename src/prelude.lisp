(define t (quote t))

(define nil ())

(define defmacro (macro (name params body)
    (list (quote define) name (list (quote macro) params body))))

(defmacro defun (name params body)
  (list (quote define) name (list (quote lambda) params body)))

(defmacro let--1 (binding body)
  (list (list (quote lambda) (list (car binding)) body)
        (car (cdr binding))))

(defun unzip (pairs)
  (if pairs
      (let--1 (fsts-snds (unzip (cdr pairs)))
              (let--1 (fsts (car fsts-snds))
                      (let--1 (snds (cdr fsts-snds))
                              (cons
                               (cons (car (car pairs)) fsts)
                               (cons (cdr (car pairs)) snds)))))
      (cons nil nil)))
