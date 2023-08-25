(define t (quote t))

(define nil ())

(define defmacro (macro (name params body)
    (list (quote define) name (list (quote macro) params body))))

(defmacro defun (name params body)
  (list (quote define) name (list (quote lambda) params body)))
