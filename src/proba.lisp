(export '(abc show-secret))

(define 'abc 123 "This is abc")

(define 'secret 789 "This is secret")

(defun -show-secret (n)
  ""
  (add secret n))

(defun show-secret ()
  "Reveale the secret!"
  ((lambda (n) (-show-secret n)) 1))
